use crate::encoding::{bitops, simple8b};
use crate::jetstream::{
    create_spatial_refs, get_delta_encoding, uvarint32, varint32, DatasetWithQuality,
    SIMPLE8B_THRESHOLD_SAMPLES, USE_GZIP_THRESHOLD_SAMPLES,
};
use flate2::read::GzDecoder;
use std::io::Read;

/// A stream protocol instance for decoding.
pub struct Decoder {
    pub id: uuid::Uuid,
    pub sampling_rate: usize,
    pub samples_per_message: usize,
    encoded_samples: usize,
    pub i32_count: usize,
    // gz_buf: bytebuffer::ByteBuffer,
    gz_buf: Vec<u8>,
    pub out: Vec<DatasetWithQuality>,
    start_timestamp: u64,
    using_simple8b: bool,
    delta_encoding_layers: usize,
    delta_sum: Vec<Vec<i32>>,
    // mutex: sync::Mutex<()>,
    use_xor: bool,
    spatial_ref: Vec<isize>,
}

impl Decoder {
    /// Creates a stream protocol decoder instance for pre-allocated output.
    pub fn new(
        id: uuid::Uuid,
        i32_count: usize,
        sampling_rate: usize,
        samples_per_message: usize,
    ) -> Self {
        // TODO make this conditional on message size to reduce memory use
        let buf_size = samples_per_message * i32_count * 8 + i32_count * 4;
        // d.gz_buf = bytebuffer::ByteBuffer::from_bytes(&Vec::with_capacity(buf_size));

        // d.delta_encoding_layers = get_delta_encoding(sampling_rate);
        let delta_encoding_layers = get_delta_encoding(sampling_rate);

        // storage for delta-delta decoding
        // d.delta_sum = vec![vec![0; i32_count]; d.delta_encoding_layers - 1];
        // d.deltaSum = make([][]int32, d.deltaEncodingLayers-1)
        // for i := range d.deltaSum {
        // 	d.deltaSum[i] = make([]int32, int32Count)
        // }

        // initialise each set of outputs in data stucture
        // for i := range d.Out {
        // d.out.iter_mut().for_each(|out| {
        //     out.i32s = vec![0; i32_count];
        //     out.q = vec![0; i32_count];
        // });

        // d.spatial_ref = vec![-1; i32_count];
        // d.spatialRef = make([]int, int32Count)
        // for i := range d.spatialRef {
        // 	d.spatialRef[i] = -1
        // }

        Self {
            id,
            sampling_rate,
            samples_per_message,
            encoded_samples: 0,
            i32_count,
            gz_buf: Vec::with_capacity(buf_size),
            out: vec![DatasetWithQuality::new(i32_count); samples_per_message],
            start_timestamp: 0,
            using_simple8b: samples_per_message > SIMPLE8B_THRESHOLD_SAMPLES,
            delta_encoding_layers,
            delta_sum: vec![vec![0; i32_count]; delta_encoding_layers - 1],
            // mutex: sync::Mutex::new(()),
            use_xor: false,
            spatial_ref: vec![-1; i32_count],
        }
    }

    /// Use XOR delta instead of arithmetic delta.
    pub fn set_xor(&mut self, xor: bool) {
        self.use_xor = xor
    }

    /// Automatically maps adjacent sets of three-phase currents for spatial compression.
    pub fn set_spatial_refs(
        &mut self,
        count: usize,
        count_v: usize,
        count_i: usize,
        include_neutral: bool,
    ) {
        self.spatial_ref = create_spatial_refs(count, count_v, count_i, include_neutral);
    }

    /// Decodes to a pre-allocated buffer.
    pub fn decode_to_buffer(&mut self, buf: &[u8], _total_length: usize) -> Result<(), String> {
        // let _lock = self.mutex.lock().unwrap();

        let mut length: usize = 16;
        // let mut _val_signed: i32 = 0;
        // let mut _val_unsigned: u32 = 0;
        // let mut _len_b: usize = 0;

        // check ID
        assert_eq!(buf[..length], self.id.as_bytes()[..], "IDs did not match");

        // decode timestamp
        // self.start_timestamp = binary.BigEndian.Uint64(buf[length..]);
        self.start_timestamp = u64::from_be_bytes(buf[length..length + 8].try_into().unwrap());
        length += 8;

        // the first timestamp is the starting value encoded in the header
        self.out[0].t = self.start_timestamp;

        // decode number of samples
        let (val_signed, len_b) = varint32(&buf[length..]);
        self.encoded_samples = val_signed as usize;
        length += len_b;

        let actual_samples = usize::min(self.encoded_samples, self.samples_per_message);

        // TODO inspect performance here
        // self.gz_buf.reset();
        self.gz_buf.clear();
        if actual_samples > USE_GZIP_THRESHOLD_SAMPLES {
            let mut gr = GzDecoder::new(&buf[length..]);

            if let Err(err) = gr.read_to_end(&mut self.gz_buf) {
                return Err(format!("gzip error: {}", err));
            }
            // gr.read_buf(&mut self.gz_buf)?;

            // let gr = gzip.NewReader(bytes.NewBuffer(&buf[length..]))?;

            // io.Copy(self.gz_buf, gr)?;
            // origLen, errRead := gr.Read((buf[length:]))
            // gr.Close();
        } else {
            // self.gz_buf = bytes.NewBuffer(buf[length..]);
            self.gz_buf = buf[length..].to_vec(); // FIXME
        }
        // log.Debug().Int("gz len", totalLength).Int64("original len", origLen).Msg("decoding")
        let out_bytes = &self.gz_buf; //.bytes();
        length = 0;

        if self.using_simple8b {
            // for simple-8b encoding, iterate through every value
            let mut decode_counter = 0;
            let mut index_ts = 0;
            let mut i = 0;

            let decoded_u64s = simple8b::for_each(
                /*buf[length:]*/ &out_bytes[length..],
                |v: u64| -> bool {
                    // manage 2D slice indices
                    index_ts = decode_counter % actual_samples;
                    if decode_counter > 0 && index_ts == 0 {
                        i += 1;
                    }

                    // get signed value back with zig-zag decoding
                    let decoded_value = bitops::zig_zag_decode64(v) as i32;

                    if index_ts == 0 {
                        self.out[index_ts].i32s[i] = decoded_value;
                    } else {
                        self.out[index_ts].t = index_ts as u64;

                        // delta decoding
                        let max_index = usize::min(index_ts, self.delta_encoding_layers - 1) - 1;
                        if self.use_xor {
                            self.delta_sum[max_index][i] ^= decoded_value;
                        } else {
                            self.delta_sum[max_index][i] += decoded_value;
                        }

                        // for k := maxIndex; k >= 1; k-- { TODO: check
                        for k in (1..=max_index).rev() {
                            if self.use_xor {
                                self.delta_sum[k - 1][i] ^= self.delta_sum[k][i];
                            } else {
                                self.delta_sum[k - 1][i] += self.delta_sum[k][i];
                            }
                        }

                        if self.use_xor {
                            self.out[index_ts].i32s[i] =
                                self.out[index_ts - 1].i32s[i] ^ self.delta_sum[0][i];
                        } else {
                            self.out[index_ts].i32s[i] =
                                self.out[index_ts - 1].i32s[i] + self.delta_sum[0][i];
                        }
                    }

                    decode_counter += 1;

                    // all variables and timesteps have been decoded
                    if decode_counter == actual_samples * self.i32_count {
                        // take care of spatial references (cannot do this piecemeal above because it disrupts the previous value history)
                        for index_ts in 0..self.out.len() {
                            for i in 0..self.out[index_ts].i32s.len() {
                                if self.spatial_ref[i] >= 0 {
                                    self.out[index_ts].i32s[i] +=
                                        self.out[index_ts].i32s[self.spatial_ref[i] as usize];
                                }
                            }
                        }

                        // stop decoding
                        return false;
                    }

                    return true;
                },
            )
            .unwrap_or(0);

            // add length of decoded unit64 blocks (8 bytes each)
            length += decoded_u64s * 8;
        } else {
            // get first set of samples using delta-delta encoding
            for i in 0..self.i32_count {
                let (val_signed, len_b) = varint32(/*buf[length:]*/ &out_bytes[length..]);
                self.out[0].i32s[i] = val_signed as i32;
                length += len_b;
            }

            // decode remaining delta-delta encoded values
            if actual_samples > 1 {
                let mut total_samples: usize = 1;
                loop {
                    // encode the sample number relative to the starting timestamp
                    self.out[total_samples].t = total_samples as u64;

                    // delta decoding
                    for i in 0..self.i32_count {
                        let (decoded_value, len_b) =
                            varint32(/*buf[length:]*/ &out_bytes[length..]);
                        length += len_b;

                        let max_index =
                            usize::min(total_samples, self.delta_encoding_layers - 1) - 1;
                        if self.use_xor {
                            self.delta_sum[max_index][i] ^= decoded_value;
                        } else {
                            self.delta_sum[max_index][i] += decoded_value;
                        }

                        // for k := maxIndex; k >= 1; k-- {
                        for k in (1..=max_index).rev() {
                            if self.use_xor {
                                self.delta_sum[k - 1][i] ^= self.delta_sum[k][i];
                            } else {
                                self.delta_sum[k - 1][i] += self.delta_sum[k][i];
                            }
                        }

                        if self.use_xor {
                            self.out[total_samples].i32s[i] =
                                self.out[total_samples - 1].i32s[i] ^ self.delta_sum[0][i];
                        } else {
                            self.out[total_samples].i32s[i] =
                                self.out[total_samples - 1].i32s[i] + self.delta_sum[0][i];
                        }
                    }
                    total_samples += 1;

                    if total_samples >= actual_samples {
                        // take care of spatial references (cannot do this piecemeal above because it disrupts the previous value history)
                        for index_ts in 0..self.out.len() {
                            for i in 0..self.out[index_ts].i32s.len() {
                                // skip the first time index
                                if self.spatial_ref[i] >= 0 {
                                    self.out[index_ts].i32s[i] +=
                                        self.out[index_ts].i32s[self.spatial_ref[i] as usize];
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
        for i in 0..self.i32_count {
            let mut sample_number = 0;
            while sample_number < actual_samples {
                let (val_unsigned, len_b) = uvarint32(/*buf[length:]*/ &out_bytes[length..]);
                length += len_b;
                self.out[sample_number].q[i] = val_unsigned as u32;

                let (val_unsigned, len_b) = uvarint32(/*buf[length:]*/ &out_bytes[length..]);
                length += len_b;

                if val_unsigned == 0 {
                    // write all remaining Q values for this variable
                    for j in sample_number + 1..self.out.len() {
                        self.out[j].q[i] = self.out[sample_number].q[i]
                    }
                    sample_number = actual_samples;
                } else {
                    // write up to val_unsigned remaining Q values for this variable
                    for j in (sample_number + 1)..(val_unsigned as usize) {
                        if sample_number < self.out.len() && j < self.out.len() {
                            self.out[j].q[i] = self.out[sample_number].q[i];
                        }
                    }
                    sample_number += val_unsigned as usize
                }
            }
        }

        for j in 0..self.delta_sum.len() {
            for i in 0..self.i32_count {
                self.delta_sum[j][i] = 0
            }
        }

        Ok(())
    }
}
