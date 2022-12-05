use crate::encoding::{bitops, simple8b};
use crate::jetstream::{
    createSpatialRefs, getDeltaEncoding, putUvarint32, putVarint32, qualityHistory, Dataset,
    DatasetWithQuality, MaxHeaderSize, Simple8bThresholdSamples, UseGzipThresholdSamples,
};
use bytebuffer::ByteBuffer;
use std::sync;
use uuid::Uuid;

type byte = u8;

// Encoder defines a stream protocol instance
pub struct Encoder {
    pub ID: Uuid,
    pub SamplingRate: usize,
    pub SamplesPerMessage: usize,
    pub Int32Count: usize,
    buf: Vec<byte>,
    bufA: Vec<byte>,
    bufB: Vec<byte>,
    outBufA: bytebuffer::Buffer,
    outBufB: bytebuffer::Buffer,
    useBufA: bool,
    len: usize,
    encodedSamples: usize,
    usingSimple8b: bool,
    deltaEncodingLayers: usize,
    simple8bValues: Vec<u64>,
    prevData: Vec<Dataset>,
    deltaN: Vec<i32>,

    qualityHistory: Vec<Vec<qualityHistory>>,
    diffs: Vec<Vec<u64>>,
    values: Vec<Vec<i32>>,
    mutex: sync::Mutex<usize>,

    useXOR: bool,
    spatialRef: Vec<isize>,
}

impl Encoder {
    /// Creates a stream protocol encoder instance.
    pub fn new(
        ID: uuid::UUID,
        int32Count: usize,
        samplingRate: usize,
        samplesPerMessage: usize,
    ) -> Self {
        // estimate maximum buffer space required
        let bufSize = MaxHeaderSize + samplesPerMessage * int32Count * 8 + int32Count * 4;

        let mut s = Self {
            ID,
            SamplingRate: samplingRate,
            SamplesPerMessage: samplesPerMessage,
            bufA: vec![0; bufSize],
            bufB: vec![0; bufSize],
            Int32Count: int32Count,
            simple8bValues: vec![0; samplesPerMessage],
        };

        // s.useXOR = true

        // initialise ping-pong buffer
        s.useBufA = true;
        s.buf = s.bufA;

        // TODO make this conditional on message size to reduce memory use
        // s.outBufA = bytes.NewBuffer(make([]byte, 0, bufSize))
        s.outBufA = ByteBuffer::from_bytes(&Vec::with_capacity(bufSize));
        s.outBufB = ByteBuffer::from_bytes(&Vec::with_capacity(bufSize));

        s.deltaEncodingLayers = getDeltaEncoding(samplingRate);

        if samplesPerMessage > Simple8bThresholdSamples {
            s.usingSimple8b = true;
            s.diffs = vec![vec![0; samplesPerMessage]; int32Count];
        // s.diffs = make([][]uint64, int32Count)
        // for i := range s.diffs {
        // 	s.diffs[i] = make([]uint64, samplesPerMessage)
        // }
        } else {
            s.values = vec![vec![0; int32Count]; samplesPerMessage];
            // s.values = make([][]int32, samplesPerMessage)
            // for i := range s.values {
            // 	s.values[i] = make([]int32, int32Count)
            // }
        }

        // storage for delta-delta encoding
        s.prevData = vec![Dataset; s.deltaEncodingLayers];
        s.prevData.iter_mut().for_each(|prevData| {
            prevData[i].Int32s = vec![0; int32Count];
        });
        s.deltaN = vec![0; s.deltaEncodingLayers];

        // s.qualityHistory = make([][]qualityHistory, int32Count)
        s.qualityHistory = vec![Vec::with_capacity(16); int32Count];
        s.qualityHistory.iter_mut().for_each(|history| {
            history.push(qualityHistory {
                value: 0,
                samples: 0,
            });
        });
        // for i := range s.qualityHistory {
        // 	// set capacity to avoid some possible allocations during encoding
        // 	s.qualityHistory[i] = make([]qualityHistory, 1, 16)
        // 	s.qualityHistory[i][0].value = 0
        // 	s.qualityHistory[i][0].samples = 0
        // }

        s.spatialRef = vec![-1; int32Count];
        // s.spatialRef = make([]int, int32Count)
        // for i := range s.spatialRef {
        // 	s.spatialRef[i] = -1
        // }

        s
    }

    /// Use XOR delta instead of arithmetic delta.
    pub fn SetXOR(&mut self, xor: bool) {
        self.useXOR = xor;
    }

    /// Automatically maps adjacent sets of three-phase currents for spatial compression.
    pub fn SetSpatialRefs(
        &mut self,
        count: usize,
        countV: usize,
        countI: usize,
        includeNeutral: bool,
    ) {
        self.spatialRef = createSpatialRefs(count, countV, countI, includeNeutral)
    }

    fn encodeSingleSample(&mut self, index: usize, value: i32) {
        if self.usingSimple8b {
            self.diffs[index][self.encodedSamples] = bitops::zig_zag_encode64(value as i64)
        } else {
            self.values[self.encodedSamples][index] = value;
        }
    }

    /// Encodes the next set of samples. It is called iteratively until the pre-defined number of samples are provided.
    pub fn Encode(&mut self, data: &mut DatasetWithQuality) -> Result<(Vec<byte>, usize), String> {
        self.mutex.lock();
        // TODO: defer s.mutex.Unlock()

        // encode header and prepare quality values
        if self.encodedSamples == 0 {
            self.len = 0;
            // self.len += copy(self.buf[self.len:], self.ID[:])
            self.buf[self.len..].copy_from_slice(self.ID);
            self.len += self.ID.as_bytes().len();

            // encode timestamp
            binary.BigEndian.PutUint64(self.buf[self.len..], data.T);
            self.len += 8;

            // record first set of quality
            // for i in 0..data.Q.len() {
            data.Q.iter().enumerate().for_each(|(i, q)| {
                self.qualityHistory[i][0].value = q;
                self.qualityHistory[i][0].samples = 1;
            });
        } else {
            // write the next quality value
            for i in 0..data.Q.len() {
                let n_hist = self.qualityHistory[i].len();
                if self.qualityHistory[i][n_hist - 1].value == data.Q[i] {
                    self.qualityHistory[i][n_hist - 1].samples += 1;
                } else {
                    self.qualityHistory[i].push(qualityHistory {
                        value: data.Q[i],
                        samples: 1,
                    });
                }
            }
        }

        for i in 0..data.Int32s.len() {
            let j = self.encodedSamples; // copy for conciseness
            let val = data.Int32s[i];

            // check if another data stream is to be used the spatial reference
            if self.spatialRef[i] >= 0 {
                val -= data.Int32s[self.spatialRef[i]]
            }

            // prepare data for delta encoding
            if j > 0 {
                if self.useXOR {
                    self.deltaN[0] = val ^ self.prevData[0].Int32s[i]
                } else {
                    self.deltaN[0] = val - self.prevData[0].Int32s[i]
                }
            }
            for k in 1..usize::min(j, self.deltaEncodingLayers) {
                if self.useXOR {
                    self.deltaN[k] = self.deltaN[k - 1] ^ self.prevData[k].Int32s[i];
                } else {
                    self.deltaN[k] = self.deltaN[k - 1] - self.prevData[k].Int32s[i];
                }
            }

            // encode the value
            if j == 0 {
                self.encodeSingleSample(i, val);
            } else {
                self.encodeSingleSample(
                    i,
                    self.deltaN[usize::min(j - 1, self.deltaEncodingLayers - 1)],
                );
            }

            // save samples and deltas for next iteration
            self.prevData[0].Int32s[i] = val;
            for k in 1..=usize::min(j, self.deltaEncodingLayers - 1) {
                self.prevData[k].Int32s[i] = self.deltaN[k - 1];
            }
        }

        self.encodedSamples += 1;
        if self.encodedSamples >= self.SamplesPerMessage {
            self.endEncode()
        } else {
            Ok((nil, 0))
        }
    }

    /// Ends the encoding early, and completes the buffer so far
    pub fn EndEncode(&mut self) -> Result<(Vec<byte>, usize), String> {
        self.mutex.lock();
        // TODO: defer s.mutex.Unlock()

        self.endEncode()
    }

    /// Ends the encoding early, but does not write to the file.
    pub fn CancelEncode(&mut self) {
        self.mutex.lock();
        // TODO: defer s.mutex.Unlock()

        // reset quality history
        self.qualityHistory.iter_mut().for_each(|history| {
            history.clear();
            history.push(qualityHistory {
                value: 0,
                samples: 0,
            });
            // s.qualityHistory[i] = s.qualityHistory[i][:1]
            // s.qualityHistory[i][0].value = 0
            // s.qualityHistory[i][0].samples = 0
        });

        // reset previous values
        self.encodedSamples = 0;
        self.len = 0;

        // send data and swap ping-pong buffer
        if self.useBufA {
            self.useBufA = false;
            self.buf = self.bufB;
        }
    }

    // internal version does not need the mutex
    fn endEncode(&mut self) -> Result<(Vec<byte>, usize), String> {
        // write encoded samples
        self.len += putVarint32(&mut self.buf[self.len..], self.encodedSamples as i32);
        let mut actualHeaderLen = self.len;

        if self.usingSimple8b {
            for i in 0..self.diffs.len() {
                // ensure slice only contains up to self.encodedSamples
                let actualSamples = usize::min(self.encodedSamples, self.SamplesPerMessage);

                let (numberOfSimple8b, _) = simple8b::EncodeAllRef(
                    &mut self.simple8bValues,
                    &self.diffs[i][..actualSamples],
                );

                // calculate efficiency of simple8b
                // multiply number of simple8b units by 2 because input is 32-bit, output is 64-bit
                // simple8bRatio := float64(2*numberOfSimple8b) / float64(actualSamples)
                // fmt.Println("simple8b efficiency:", simple8bRatio)

                for j in 0..numberOfSimple8b {
                    binary
                        .BigEndian
                        .PutUint64(self.buf[self.len..], self.simple8bValues[j]);
                    self.len += 8;
                }
            }
        } else {
            for i in 0..self.encodedSamples {
                for j in 0..self.Int32Count {
                    self.len += putVarint32(&mut self.buf[self.len..], self.values[i][j])
                }
            }
        }

        // encode final quality values using RLE
        // for i := range self.qualityHistory {
        self.qualityHistory.iter_mut().for_each(|history| {
            let n_sample = len(self.qualityHistory[i]);
            // override final number of samples to zero
            history[n_sample - 1].samples = 0; // TODO: .last

            // otherwise, encode each value
            // for j := range self.qualityHistory[i] {
            history.iter().for_each(|sample| {
                self.len += putUvarint32(&mut self.buf[self.len..], sample.value);
                self.len += putUvarint32(&mut self.buf[self.len..], sample.samples);
            });
        });

        // reset quality history
        // for i := range self.qualityHistory {
        self.qualityHistory.iter_mut().for_each(|history| {
            // self.qualityHistory[i] = self.qualityHistory[i][:1]
            // self.qualityHistory[i][0].value = 0
            // self.qualityHistory[i][0].samples = 0
            history.clear();
            history.push(qualityHistory {
                value: 0,
                samples: 0,
            });
        });

        // experiment with Huffman coding
        // var enc huff0.Scratch
        // comp, _, err := huff0.Compress4X(self.buf[0:self.len], &enc)
        // if err == huff0.ErrIncompressible || err == huff0.ErrUseRLE || err == huff0.ErrTooBig {
        // 	log.Error().Err(err).Msg("huff0 error")
        // }
        // log.Debug().Int("huff0 len", len(comp)).Int("original len", self.len).Msg("huff0 output")

        // experiment with gzip
        // TODO determine if bufA/bufB can be replaced with this internal double buffering
        let activeOutBuf = if !self.useBufA {
            self.outBufB
        } else {
            self.outBufA
        };

        // TODO inspect performance here
        activeOutBuf.Reset();
        if self.encodedSamples > UseGzipThresholdSamples {
            // do not compress header
            activeOutBuf.Write(self.buf[..actualHeaderLen]);

            let (gz, _) = gzip.NewWriterLevel(activeOutBuf, gzip.BestCompression); // can test entropy coding by using gzip.HuffmanOnly
            if let Err(err) = gz.Write(self.buf[actualHeaderLen..self.len]) {
                log.Error().Err(err).Msg("could not write gz");
            }
            if let Err(err) = gz.Close() {
                log.Error().Err(err).Msg("could not close gz");
            };

            // ensure that gzip size is never greater that input for all input sizes
            if activeOutBuf.Len() > self.len && self.encodedSamples == self.SamplesPerMessage {
                log.Error()
                    .Int("gz", activeOutBuf.Len())
                    .Int("original", self.len)
                    .Int("SamplesPerMessage", self.SamplesPerMessage)
                    .Msg("gzip encoding length greater")
            }
        } else {
            activeOutBuf.Write(self.buf[..self.len]);
        }

        // reset previous values
        // finalLen = self.len
        self.encodedSamples = 0;
        self.len = 0;

        // send data and swap ping-pong buffer
        if self.useBufA {
            self.useBufA = false;
            self.buf = self.bufB;
            return Ok((activeOutBuf.Bytes(), activeOutBuf.Len()));
            // return Ok((self.bufA[..finalLen], finalLen))
        }

        self.useBufA = true;
        self.buf = self.bufA;
        Ok((activeOutBuf.Bytes(), activeOutBuf.Len()))
        // return self.bufB[0:finalLen], finalLen, nil
    }
}
