use crate::encoding::{bitops, simple8b};
use crate::jetstream::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{as_error, error};
use std::io::Write;
use uuid::Uuid;

/// Encoder defines a stream protocol instance
pub struct Encoder {
    pub id: Uuid,
    pub sampling_rate: usize,
    pub samples_per_message: usize,
    pub i32_count: usize,
    // buf: Vec<u8>,
    buf_a: Vec<u8>,
    buf_b: Vec<u8>,
    // out_buf_a: bytebuffer::ByteBuffer,
    // out_buf_b: bytebuffer::ByteBuffer,
    use_buf_a: bool,
    len: usize,
    encoded_samples: usize,
    using_simple8b: bool,
    delta_encoding_layers: usize,
    simple8b_values: Vec<u64>,
    prev_data: Vec<Dataset>,
    delta_n: Vec<i32>,

    quality_history: Vec<Vec<QualityHistory>>,
    diffs: Vec<Vec<u64>>,
    values: Vec<Vec<i32>>,
    // mutex: sync::Mutex<()>,
    use_xor: bool,
    spatial_ref: Vec<Option<usize>>,
}

impl Encoder {
    /// Creates a stream protocol encoder instance.
    pub fn new(
        id: uuid::Uuid,
        i32_count: usize,
        sampling_rate: usize,
        samples_per_message: usize,
    ) -> Self {
        // estimate maximum buffer space required
        let buf_size = MAX_HEADER_SIZE + samples_per_message * i32_count * 8 + i32_count * 4;

        // s.use_xor = true

        // initialise ping-pong buffer
        // s.use_buf_a = true;
        // s.buf = s.buf_a;

        // TODO make this conditional on message size to reduce memory use
        // s.out_buf_a = bytes.NewBuffer(make([]byte, 0, buf_size))
        // s.out_buf_a = ByteBuffer::from_bytes(&Vec::with_capacity(buf_size));
        // s.out_buf_b = ByteBuffer::from_bytes(&Vec::with_capacity(buf_size));

        let delta_encoding_layers = get_delta_encoding(sampling_rate);

        let using_simple8b = samples_per_message > SIMPLE8B_THRESHOLD_SAMPLES;

        // if samples_per_message > SIMPLE8B_THRESHOLD_SAMPLES {
        //     s.using_simple8b = true;
        //     s.diffs = vec![vec![0; samples_per_message]; i32_count];
        // // s.diffs = make([][]uint64, int32count)
        // // for i := range s.diffs {
        // // 	s.diffs[i] = make([]uint64, samples_per_message)
        // // }
        // } else {
        //     s.values = vec![vec![0; i32_count]; samples_per_message];
        //     // s.values = make([][]int32, samples_per_message)
        //     // for i := range s.values {
        //     // 	s.values[i] = make([]int32, int32count)
        //     // }
        // }

        // storage for delta-delta encoding
        // s.prev_data = vec![Dataset::new(i32_count); s.delta_encoding_layers];
        // s.prev_data.iter_mut().for_each(|prev_data| {
        //     prev_data[i].Int32s = vec![0; i32_count];
        // });
        // s.delta_n = vec![0; s.delta_encoding_layers];

        // s.quality_history = make([][]quality_history, int32count)
        // for i := range s.quality_history {
        // 	// set capacity to avoid some possible allocations during encoding
        // 	s.quality_history[i] = make([]quality_history, 1, 16)
        // 	s.quality_history[i][0].value = 0
        // 	s.quality_history[i][0].samples = 0
        // }
        // let mut quality_history = vec![Vec::with_capacity(16); i32_count];
        // quality_history.iter_mut().for_each(|history| {
        //     history.push(QualityHistory {
        //         value: 0,
        //         samples: 0,
        //     });
        // });

        // s.spatial_ref = vec![-1; i32_count];
        // s.spatial_ref = make([]int, int32count)
        // for i := range s.spatial_ref {
        // 	s.spatial_ref[i] = -1
        // }

        Self {
            id,
            sampling_rate,
            samples_per_message,
            i32_count,

            buf_a: vec![0; buf_size],
            buf_b: vec![0; buf_size],

            // out_buf_a: ByteBuffer::from_bytes(&Vec::with_capacity(buf_size)),
            // out_buf_b: ByteBuffer::from_bytes(&Vec::with_capacity(buf_size)),

            // initialise ping-pong buffer
            use_buf_a: true,
            // buf: buf_a,
            len: 0,
            encoded_samples: 0,
            using_simple8b,
            delta_encoding_layers,

            simple8b_values: vec![0; samples_per_message],
            prev_data: vec![Dataset::new(i32_count); delta_encoding_layers],
            delta_n: vec![0; delta_encoding_layers],

            quality_history: vec![vec![QualityHistory::default()]; i32_count],
            diffs: if using_simple8b {
                vec![vec![0; samples_per_message]; i32_count]
            } else {
                vec![]
            },
            values: if !using_simple8b {
                vec![vec![0; i32_count]; samples_per_message]
            } else {
                vec![]
            },
            // mutex: sync::Mutex::new(()),
            use_xor: false,
            spatial_ref: vec![None; i32_count],
        }
    }

    fn buf(&self) -> &Vec<u8> {
        if self.use_buf_a {
            &self.buf_a
        } else {
            &self.buf_b
        }
    }

    fn buf_mut(&mut self) -> &mut Vec<u8> {
        if self.use_buf_a {
            &mut self.buf_a
        } else {
            &mut self.buf_b
        }
    }

    /// Use XOR delta instead of arithmetic delta.
    pub fn set_xor(&mut self, xor: bool) {
        self.use_xor = xor;
    }

    /// Automatically maps adjacent sets of three-phase currents for spatial compression.
    pub fn set_spatial_refs(
        &mut self,
        count: usize,
        count_v: usize,
        count_i: usize,
        include_neutral: bool,
    ) {
        self.spatial_ref = create_spatial_refs(count, count_v, count_i, include_neutral)
    }

    fn encode_single_sample(&mut self, index: usize, value: i32) {
        if self.using_simple8b {
            self.diffs[index][self.encoded_samples] = bitops::zig_zag_encode64(value as i64)
        } else {
            self.values[self.encoded_samples][index] = value;
        }
    }

    /// Encodes the next set of samples. It is called iteratively until the pre-defined number of samples are provided.
    pub fn encode(&mut self, data: &mut DatasetWithQuality) -> Result<(Vec<u8>, usize), String> {
        // let _lock = self.mutex.lock().unwrap();

        // encode header and prepare quality values
        if self.encoded_samples == 0 {
            // self.len = 0;
            // self.len += copy(self.buf[self.len:], self.ID[:])
            let id_bytes = self.id.as_bytes().clone();
            self.buf_mut()[0..16].copy_from_slice(&id_bytes);
            // self.len += self.id.as_bytes().len();
            // self.len += id_bytes.len();
            self.len = 16;

            // encode timestamp
            // binary.BigEndian.PutUint64(self.buf[self.len..], data.t);
            // self.buf[self.len..] = data.t.to_be_bytes(); // TODO: double check
            let len = self.len;
            self.buf_mut()[len..len + 8].copy_from_slice(&data.t.to_be_bytes());
            self.len += 8;

            // record first set of quality
            // for i in 0..data.Q.len() {
            data.q.iter().enumerate().for_each(|(i, &q)| {
                self.quality_history[i][0].value = q;
                self.quality_history[i][0].samples = 1;
            });
        } else {
            // write the next quality value
            for i in 0..data.q.len() {
                // let len = self.quality_history[i].len();
                // if self.quality_history[i][len - 1].value == data.q[i] {
                //     self.quality_history[i][len - 1].samples += 1;
                if self.quality_history[i].last().unwrap().value == data.q[i] {
                    self.quality_history[i].last_mut().unwrap().samples += 1;
                } else {
                    self.quality_history[i].push(QualityHistory {
                        value: data.q[i],
                        samples: 1,
                    });
                }
            }
        }

        for i in 0..data.i32s.len() {
            let j = self.encoded_samples; // copy for conciseness
            let mut val = data.i32s[i];

            // check if another data stream is to be used the spatial reference
            if let Some(spatial_ref_i) = self.spatial_ref[i] {
                val -= data.i32s[spatial_ref_i];
            }

            // prepare data for delta encoding
            if j > 0 {
                if self.use_xor {
                    self.delta_n[0] = val ^ self.prev_data[0].i32s[i];
                } else {
                    self.delta_n[0] = val - self.prev_data[0].i32s[i];
                }
            }
            for k in 1..usize::min(j, self.delta_encoding_layers) {
                if self.use_xor {
                    self.delta_n[k] = self.delta_n[k - 1] ^ self.prev_data[k].i32s[i];
                } else {
                    self.delta_n[k] = self.delta_n[k - 1] - self.prev_data[k].i32s[i];
                }
            }

            // encode the value
            if j == 0 {
                self.encode_single_sample(i, val);
            } else {
                self.encode_single_sample(
                    i,
                    self.delta_n[usize::min(j - 1, self.delta_encoding_layers - 1)],
                );
            }

            // save samples and deltas for next iteration
            self.prev_data[0].i32s[i] = val;
            for k in 1..=usize::min(j, self.delta_encoding_layers - 1) {
                self.prev_data[k].i32s[i] = self.delta_n[k - 1];
            }
        }

        self.encoded_samples += 1;
        if self.encoded_samples >= self.samples_per_message {
            self._end_encode()
        } else {
            Ok((vec![], 0))
        }
    }

    /// Ends the encoding early, and completes the buffer so far
    pub fn end_encode(&mut self) -> Result<(Vec<u8>, usize), String> {
        // let _lock = self.mutex.lock().unwrap();

        self._end_encode()
    }

    /// Ends the encoding early, but does not write to the file.
    pub fn cancel_encode(&mut self) {
        // let _lock = self.mutex.lock().unwrap();

        // reset quality history
        self.quality_history = vec![vec![QualityHistory::default()]; self.i32_count];
        // self.quality_history.iter_mut().for_each(|history| {
        //     history.clear();
        //     history.push(QualityHistory {
        //         value: 0,
        //         samples: 0,
        //     });
        //     // s.quality_history[i] = s.quality_history[i][:1]
        //     // s.quality_history[i][0].value = 0
        //     // s.quality_history[i][0].samples = 0
        // });

        // reset previous values
        self.encoded_samples = 0;
        self.len = 0;

        // send data and swap ping-pong buffer
        if self.use_buf_a {
            self.use_buf_a = false;
            // self.buf = self.buf_b;
        }
    }

    // internal version does not need the mutex
    fn _end_encode(&mut self) -> Result<(Vec<u8>, usize), String> {
        // write encoded samples
        let len = self.len;
        let encoded_samples = self.encoded_samples as i32;
        self.len += put_varint32(&mut self.buf_mut()[len..], encoded_samples as i32);
        let actual_header_len = self.len;

        if self.using_simple8b {
            for i in 0..self.diffs.len() {
                // ensure slice only contains up to self.encoded_samples
                let actual_samples = usize::min(self.encoded_samples, self.samples_per_message);

                let number_of_simple8b = simple8b::encode_all_ref(
                    &mut self.simple8b_values,
                    &self.diffs[i][..actual_samples],
                )
                .unwrap_or(0);

                // calculate efficiency of simple8b
                // multiply number of simple8b units by 2 because input is 32-bit, output is 64-bit
                // simple8bRatio := float64(2*number_of_simple8b) / float64(actual_samples)
                // fmt.Println("simple8b efficiency:", simple8bRatio)

                for j in 0..number_of_simple8b {
                    // binary.BigEndian.PutUint64(self.buf[self.len..], self.simple8b_values[j]);
                    let len = self.len;
                    let simple8b_values = self.simple8b_values[j].to_be_bytes();
                    self.buf_mut()[len..len + 8].copy_from_slice(&simple8b_values);
                    // self.buf[self.len..] = *self.simple8b_values[j].to_be_bytes(); // TODO: double check
                    self.len += 8;
                }
            }
        } else {
            for i in 0..self.encoded_samples {
                for j in 0..self.i32_count {
                    let len = self.len;
                    let value = self.values[i][j];
                    self.len += put_varint32(&mut self.buf_mut()[len..], value);
                }
            }
        }

        // encode final quality values using RLE
        for i in 0..self.quality_history.len() {
            // self.quality_history.iter_mut().for_each(|history| {
            // override final number of samples to zero
            // let n_sample = self.quality_history[i].len();
            // history[n_sample - 1].samples = 0;
            self.quality_history[i].last_mut().unwrap().samples = 0;

            // otherwise, encode each value
            for j in 0..self.quality_history[i].len() {
                let (len, value) = (self.len, self.quality_history[i][j].value);
                self.len += put_uvarint32(&mut self.buf_mut()[len..], value);

                let (len, samples) = (self.len, self.quality_history[i][j].samples);
                self.len += put_uvarint32(&mut self.buf_mut()[len..], samples);
            }
        }

        // reset quality history
        self.quality_history = vec![vec![QualityHistory::default()]; self.i32_count];
        // for i := range self.quality_history {
        // self.quality_history.iter_mut().for_each(|history| {
        //     // self.quality_history[i] = self.quality_history[i][:1]
        //     // self.quality_history[i][0].value = 0
        //     // self.quality_history[i][0].samples = 0
        //     history.clear();
        //     history.push(QualityHistory {
        //         value: 0,
        //         samples: 0,
        //     });
        // });

        // experiment with Huffman coding
        // var enc huff0.Scratch
        // comp, _, err := huff0.Compress4X(self.buf[0:self.len], &enc)
        // if err == huff0.ErrIncompressible || err == huff0.ErrUseRLE || err == huff0.ErrTooBig {
        // 	log.Error().Err(err).Msg("huff0 error")
        // }
        // log.Debug().Int("huff0 len", len(comp)).Int("original len", self.len).Msg("huff0 output")

        // experiment with gzip
        // TODO determine if buf_a/buf_b can be replaced with this internal double buffering
        // let active_out_buf = if !self.use_buf_a {
        //     &mut self.out_buf_b
        // } else {
        //     &mut self.out_buf_a
        // };

        // TODO inspect performance here
        // active_out_buf.clear();
        let out_buf = if self.encoded_samples > USE_GZIP_THRESHOLD_SAMPLES {
            // do not compress header
            // active_out_buf.write(&self.buf()[..actual_header_len]);
            // let header = &self.buf()[..actual_header_len];
            // active_out_buf.write(header);
            let out_buf = self.buf()[..actual_header_len].to_vec();

            // let (gz, _) = gzip.NewWriterLevel(active_out_buf, gzip.BestCompression); // can test entropy coding by using gzip.HuffmanOnly
            let mut gz = GzEncoder::new(out_buf, Compression::best());
            if let Err(err) = gz.write_all(&self.buf()[actual_header_len..self.len]) {
                error!(err = as_error!(err); "could not write gz");
            }
            // if let Err(err) = gz.finish() {
            //     error!(err = as_error!(err); "could not close gz");
            // };

            match gz.finish() {
                Err(err) => {
                    error!(err = as_error!(err); "could not close gz");
                    vec![]
                }
                Ok(out_buf) => {
                    // ensure that gzip size is never greater that input for all input sizes
                    if out_buf.len() > self.len && self.encoded_samples == self.samples_per_message
                    {
                        error!(
                            gz = out_buf.len(),
                            original = self.len,
                            samples_per_message = self.samples_per_message;
                            "gzip encoding length greater"
                        );
                    }
                    out_buf
                }
            }
        } else {
            // active_out_buf.write(&self.buf()[..self.len]);
            self.buf()[..self.len].to_vec()
        };

        // reset previous values
        // finalLen = self.len
        self.encoded_samples = 0;
        self.len = 0;

        // send data and swap ping-pong buffer
        // let out_bytes = out_buf.as_bytes().to_vec();
        if self.use_buf_a {
            self.use_buf_a = false;
            // self.buf = self.buf_b;
            // Ok((active_out_buf.as_bytes().to_vec(), active_out_buf.len()))
            // return Ok((self.buf_a[..finalLen], finalLen))
        } else {
            self.use_buf_a = true;
            // self.buf = self.buf_a;
            // Ok((active_out_buf.as_bytes().to_vec(), active_out_buf.len()))
            // return self.buf_b[0:finalLen], finalLen, nil
        }
        let len = out_buf.len();
        Ok((out_buf, len))
    }
}
