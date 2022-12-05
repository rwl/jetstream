use crate::encoding::simple8b;
use crate::jetstream::{
    createSpatialRefs, getDeltaEncoding, uvarint32, varint32, DatasetWithQuality,
    Simple8bThresholdSamples, UseGzipThresholdSamples,
};
use std::sync;

/// A stream protocol instance for decoding.
pub struct Decoder {
    pub ID: uuid::Uuid,
    pub SamplingRate: usize,
    pub SamplesPerMessage: usize,
    encodedSamples: usize,
    pub Int32Count: usize,
    gzBuf: bytebuffer::Buffer,
    pub Out: Vec<DatasetWithQuality>,
    startTimestamp: u64,
    usingSimple8b: bool,
    deltaEncodingLayers: usize,
    deltaSum: Vec<Vec<i32>>,
    mutex: sync::Mutex<usize>,

    useXOR: bool,
    spatialRef: Vec<isize>,
}

impl Decoder {
    /// Creates a stream protocol decoder instance for pre-allocated output.
    pub fn new(
        ID: uuid::Uuid,
        int32Count: usize,
        samplingRate: usize,
        samplesPerMessage: usize,
    ) -> Self {
        let mut d = Self {
            ID: ID,
            Int32Count: int32Count,
            SamplingRate: samplingRate,
            SamplesPerMessage: samplesPerMessage,
            Out: vec![DatasetWithQuality; samplesPerMessage],
        };

        // d.useXOR = true

        if samplesPerMessage > Simple8bThresholdSamples {
            d.usingSimple8b = true;
        }

        // TODO make this conditional on message size to reduce memory use
        let bufSize = samplesPerMessage * int32Count * 8 + int32Count * 4;
        d.gzBuf = bytebuffer::ByteBuffer::from_bytes(&Vec::with_capacity(bufSize));

        d.deltaEncodingLayers = getDeltaEncoding(samplingRate);

        // storage for delta-delta decoding
        d.deltaSum = vec![vec![0; int32Count]; d.deltaEncodingLayers - 1];
        // d.deltaSum = make([][]int32, d.deltaEncodingLayers-1)
        // for i := range d.deltaSum {
        // 	d.deltaSum[i] = make([]int32, int32Count)
        // }

        // initialise each set of outputs in data stucture
        // for i := range d.Out {
        d.Out.iter_mut().for_each(|out| {
            out.Int32s = vec![0; int32Count];
            out.Q = vec![0; int32Count];
        });

        d.spatialRef = vec![-1; int32Count];
        // d.spatialRef = make([]int, int32Count)
        // for i := range d.spatialRef {
        // 	d.spatialRef[i] = -1
        // }

        d
    }

    /// Use XOR delta instead of arithmetic delta.
    pub fn SetXOR(&mut self, xor: bool) {
        self.useXOR = xor
    }

    /// Automatically maps adjacent sets of three-phase currents for spatial compression.
    pub fn SetSpatialRefs(
        &mut self,
        count: usize,
        countV: usize,
        countI: usize,
        includeNeutral: bool,
    ) {
        self.spatialRef = createSpatialRefs(count, countV, countI, includeNeutral);
    }

    // DecodeToBuffer decodes to a pre-allocated buffer
    pub fn DecodeToBuffer(&mut self, buf: &[u8], totalLength: usize) -> Result<(), String> {
        self.mutex.lock();
        // TODO: defer self.mutex.Unlock()

        let mut length: usize = 16;
        let mut valSigned: i32 = 0;
        let mut valUnsigned: u32 = 0;
        let mut lenB: usize = 0;

        // check ID
        let res = bytes.Compare(buf[..length], self.ID[..]);
        if res != 0 {
            return Err("IDs did not match".to_string());
        }

        // decode timestamp
        self.startTimestamp = binary.BigEndian.Uint64(buf[length..]);
        length += 8;

        // the first timestamp is the starting value encoded in the header
        self.Out[0].T = self.startTimestamp;

        // decode number of samples
        let (valSigned, lenB) = varint32(&buf[length..]);
        self.encodedSamples = valSigned as usize;
        length += lenB;

        let actualSamples = usize::min(self.encodedSamples, self.SamplesPerMessage);

        // TODO inspect performance here
        self.gzBuf.Reset();
        if actualSamples > UseGzipThresholdSamples {
            let gr = gzip.NewReader(bytes.NewBuffer(&buf[length..]))?;

            io.Copy(self.gzBuf, gr)?;
            // origLen, errRead := gr.Read((buf[length:]))
            gr.Close();
        } else {
            self.gzBuf = bytes.NewBuffer(buf[length..]);
        }
        // log.Debug().Int("gz len", totalLength).Int64("original len", origLen).Msg("decoding")
        let outBytes = self.gzBuf.Bytes();
        length = 0;

        if self.usingSimple8b {
            // for simple-8b encoding, iterate through every value
            let mut decodeCounter = 0;
            let mut indexTs = 0;
            let mut i = 0;

            let (decodedUnit64s, _) =
                simple8b::ForEach(/*buf[length:]*/ outBytes[length..], |v: u64| -> bool {
                    // manage 2D slice indices
                    indexTs = decodeCounter % actualSamples;
                    if decodeCounter > 0 && indexTs == 0 {
                        i += 1;
                    }

                    // get signed value back with zig-zag decoding
                    let decodedValue = bitops.ZigZagDecode64(v) as i32;

                    if indexTs == 0 {
                        self.Out[indexTs].Int32s[i] = decodedValue;
                    } else {
                        self.Out[indexTs].T = indexTs as u64;

                        // delta decoding
                        let maxIndex = usize::min(indexTs, self.deltaEncodingLayers - 1) - 1;
                        if self.useXOR {
                            self.deltaSum[maxIndex][i] ^= decodedValue;
                        } else {
                            self.deltaSum[maxIndex][i] += decodedValue;
                        }

                        // for k := maxIndex; k >= 1; k-- { TODO: check
                        for k in (1..=maxIndex).rev() {
                            if self.useXOR {
                                self.deltaSum[k - 1][i] ^= self.deltaSum[k][i];
                            } else {
                                self.deltaSum[k - 1][i] += self.deltaSum[k][i];
                            }
                        }

                        if self.useXOR {
                            self.Out[indexTs].Int32s[i] =
                                self.Out[indexTs - 1].Int32s[i] ^ self.deltaSum[0][i];
                        } else {
                            self.Out[indexTs].Int32s[i] =
                                self.Out[indexTs - 1].Int32s[i] + self.deltaSum[0][i];
                        }
                    }

                    decodeCounter += 1;

                    // all variables and timesteps have been decoded
                    if decodeCounter == actualSamples * self.Int32Count {
                        // take care of spatial references (cannot do this piecemeal above because it disrupts the previous value history)
                        for indexTs in 0..self.Out.len() {
                            for i in 0..self.Out[indexTs].Int32s.len() {
                                if self.spatialRef[i] >= 0 {
                                    self.Out[indexTs].Int32s[i] +=
                                        self.Out[indexTs].Int32s[self.spatialRef[i]];
                                }
                            }
                        }

                        // stop decoding
                        return false;
                    }

                    return true;
                });

            // add length of decoded unit64 blocks (8 bytes each)
            length += decodedUnit64s * 8;
        } else {
            // get first set of samples using delta-delta encoding
            for i in 0..self.Int32Count {
                let (valSigned, lenB) = varint32(/*buf[length:]*/ outBytes[length..]);
                self.Out[0].Int32s[i] = int32(valSigned);
                length += lenB;
            }

            // decode remaining delta-delta encoded values
            if actualSamples > 1 {
                let mut totalSamples: usize = 1;
                loop {
                    // encode the sample number relative to the starting timestamp
                    self.Out[totalSamples].T = totalSamples as u64;

                    // delta decoding
                    for i in 0..self.Int32Count {
                        let (decodedValue, lenB) =
                            varint32(/*buf[length:]*/ outBytes[length..]);
                        length += lenB;

                        let maxIndex = usize::min(totalSamples, self.deltaEncodingLayers - 1) - 1;
                        if self.useXOR {
                            self.deltaSum[maxIndex][i] ^= decodedValue;
                        } else {
                            self.deltaSum[maxIndex][i] += decodedValue;
                        }

                        // for k := maxIndex; k >= 1; k-- {
                        for k in (1..=maxIndex).rev() {
                            if self.useXOR {
                                self.deltaSum[k - 1][i] ^= self.deltaSum[k][i];
                            } else {
                                self.deltaSum[k - 1][i] += self.deltaSum[k][i];
                            }
                        }

                        if self.useXOR {
                            self.Out[totalSamples].Int32s[i] =
                                self.Out[totalSamples - 1].Int32s[i] ^ self.deltaSum[0][i];
                        } else {
                            self.Out[totalSamples].Int32s[i] =
                                self.Out[totalSamples - 1].Int32s[i] + self.deltaSum[0][i];
                        }
                    }
                    totalSamples += 1;

                    if totalSamples >= actualSamples {
                        // take care of spatial references (cannot do this piecemeal above because it disrupts the previous value history)
                        for indexTs in 0..self.Out.len() {
                            for i in 0..self.Out[indexTs].Int32s.len() {
                                // skip the first time index
                                if self.spatialRef[i] >= 0 {
                                    self.Out[indexTs].Int32s[i] +=
                                        self.Out[indexTs].Int32s[self.spatialRef[i]];
                                }
                            }
                        }

                        // end decoding
                        break;
                    }
                }
            }
        }

        // populate quality structure
        for i in 0..self.Int32Count {
            let mut sampleNumber = 0;
            while sampleNumber < actualSamples {
                let (valUnsigned, lenB) = uvarint32(/*buf[length:]*/ outBytes[length..]);
                length += lenB;
                self.Out[sampleNumber].Q[i] = valUnsigned as u32;

                let (valUnsigned, lenB) = uvarint32(/*buf[length:]*/ outBytes[length..]);
                length += lenB;

                if valUnsigned == 0 {
                    // write all remaining Q values for this variable
                    for j in sampleNumber + 1..self.Out.len() {
                        self.Out[j].Q[i] = self.Out[sampleNumber].Q[i]
                    }
                    sampleNumber = actualSamples;
                } else {
                    // write up to valUnsigned remaining Q values for this variable
                    for j in (sampleNumber + 1)..(valUnsigned as usize) {
                        if sampleNumber < self.Out.len() && j < self.Out.len() {
                            self.Out[j].Q[i] = self.Out[sampleNumber].Q[i];
                        }
                    }
                    sampleNumber += valUnsigned as usize
                }
            }
        }

        for j in 0..self.deltaSum.len() {
            for i in 0..self.Int32Count {
                self.deltaSum[j][i] = 0
            }
        }

        Ok(())
    }
}
