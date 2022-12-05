use crate::encoding::{bitops, simple8b};
use crate::jetstream::*;
use bytebuffer::ByteBuffer;
use std::sync;
use uuid::Uuid;

type byte = u8;

// Encoder defines a stream protocol instance
pub struct Encoder {
    pub id: Uuid,
    pub sampling_rate: usize,
    pub samples_per_message: usize,
    pub i32_count: usize,
    buf: Vec<byte>,
    buf_a: Vec<byte>,
    buf_b: Vec<byte>,
    out_buf_a: bytebuffer::Buffer,
    out_buf_b: bytebuffer::Buffer,
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
    mutex: sync::Mutex<usize>,

    use_xor: bool,
    spatial_ref: Vec<isize>,
}

impl Encoder {
    /// Creates a stream protocol encoder instance.
    pub fn new(
        id: uuid::UUID,
        i32_count: usize,
        sampling_rate: usize,
        samples_per_message: usize,
    ) -> Self {
        // estimate maximum buffer space required
        let buf_size = MAX_HEADER_SIZE + samples_per_message * i32_count * 8 + i32_count * 4;

        let mut s = Self {
            id,
            sampling_rate,
            samples_per_message,
            buf_a: vec![0; buf_size],
            buf_b: vec![0; buf_size],
            i32_count,
            simple8b_values: vec![0; samples_per_message],
        };

        // s.use_xor = true

        // initialise ping-pong buffer
        s.use_buf_a = true;
        s.buf = s.buf_a;

        // TODO make this conditional on message size to reduce memory use
        // s.out_buf_a = bytes.NewBuffer(make([]byte, 0, buf_size))
        s.out_buf_a = ByteBuffer::from_bytes(&Vec::with_capacity(buf_size));
        s.out_buf_b = ByteBuffer::from_bytes(&Vec::with_capacity(buf_size));

        s.delta_encoding_layers = get_delta_encoding(sampling_rate);

        if samples_per_message > SIMPLE8B_THRESHOLD_SAMPLES {
            s.using_simple8b = true;
            s.diffs = vec![vec![0; samples_per_message]; i32_count];
        // s.diffs = make([][]uint64, int32count)
        // for i := range s.diffs {
        // 	s.diffs[i] = make([]uint64, samples_per_message)
        // }
        } else {
            s.values = vec![vec![0; i32_count]; samples_per_message];
            // s.values = make([][]int32, samples_per_message)
            // for i := range s.values {
            // 	s.values[i] = make([]int32, int32count)
            // }
        }

        // storage for delta-delta encoding
        s.prev_data = vec![Dataset; s.delta_encoding_layers];
        s.prev_data.iter_mut().for_each(|prev_data| {
            prev_data[i].Int32s = vec![0; i32_count];
        });
        s.delta_n = vec![0; s.delta_encoding_layers];

        // s.quality_history = make([][]quality_history, int32count)
        s.quality_history = vec![Vec::with_capacity(16); i32_count];
        s.quality_history.iter_mut().for_each(|history| {
            history.push(QualityHistory {
                value: 0,
                samples: 0,
            });
        });
        // for i := range s.quality_history {
        // 	// set capacity to avoid some possible allocations during encoding
        // 	s.quality_history[i] = make([]quality_history, 1, 16)
        // 	s.quality_history[i][0].value = 0
        // 	s.quality_history[i][0].samples = 0
        // }

        s.spatial_ref = vec![-1; i32_count];
        // s.spatial_ref = make([]int, int32count)
        // for i := range s.spatial_ref {
        // 	s.spatial_ref[i] = -1
        // }

        s
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
    pub fn encode(&mut self, data: &mut DatasetWithQuality) -> Result<(Vec<byte>, usize), String> {
        self.mutex.lock();
        // TODO: defer s.mutex.Unlock()

        // encode header and prepare quality values
        if self.encoded_samples == 0 {
            self.len = 0;
            // self.len += copy(self.buf[self.len:], self.ID[:])
            self.buf[self.len..].copy_from_slice(self.id);
            self.len += self.id.as_bytes().len();

            // encode timestamp
            binary.BigEndian.PutUint64(self.buf[self.len..], data.t);
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
                let n_hist = self.quality_history[i].len();
                if self.quality_history[i][n_hist - 1].value == data.q[i] {
                    self.quality_history[i][n_hist - 1].samples += 1;
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
            if self.spatial_ref[i] >= 0 {
                val -= data.i32s[self.spatial_ref[i]]
            }

            // prepare data for delta encoding
            if j > 0 {
                if self.use_xor {
                    self.delta_n[0] = val ^ self.prev_data[0].i32s[i]
                } else {
                    self.delta_n[0] = val - self.prev_data[0].i32s[i]
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
            Ok((nil, 0))
        }
    }

    /// Ends the encoding early, and completes the buffer so far
    pub fn end_encode(&mut self) -> Result<(Vec<byte>, usize), String> {
        self.mutex.lock();
        // TODO: defer s.mutex.Unlock()

        self._end_encode()
    }

    /// Ends the encoding early, but does not write to the file.
    pub fn cancel_encode(&mut self) {
        self.mutex.lock();
        // TODO: defer s.mutex.Unlock()

        // reset quality history
        self.quality_history.iter_mut().for_each(|history| {
            history.clear();
            history.push(QualityHistory {
                value: 0,
                samples: 0,
            });
            // s.quality_history[i] = s.quality_history[i][:1]
            // s.quality_history[i][0].value = 0
            // s.quality_history[i][0].samples = 0
        });

        // reset previous values
        self.encoded_samples = 0;
        self.len = 0;

        // send data and swap ping-pong buffer
        if self.use_buf_a {
            self.use_buf_a = false;
            self.buf = self.buf_b;
        }
    }

    // internal version does not need the mutex
    fn _end_encode(&mut self) -> Result<(Vec<byte>, usize), String> {
        // write encoded samples
        self.len += put_varint32(&mut self.buf[self.len..], self.encoded_samples as i32);
        let mut actual_header_len = self.len;

        if self.using_simple8b {
            for i in 0..self.diffs.len() {
                // ensure slice only contains up to self.encoded_samples
                let actual_samples = usize::min(self.encoded_samples, self.samples_per_message);

                let (number_of_simple8b, _) = simple8b::encode_all_ref(
                    &mut self.simple8b_values,
                    &self.diffs[i][..actual_samples],
                );

                // calculate efficiency of simple8b
                // multiply number of simple8b units by 2 because input is 32-bit, output is 64-bit
                // simple8bRatio := float64(2*number_of_simple8b) / float64(actual_samples)
                // fmt.Println("simple8b efficiency:", simple8bRatio)

                for j in 0..number_of_simple8b {
                    binary
                        .BigEndian
                        .PutUint64(self.buf[self.len..], self.simple8b_values[j]);
                    self.len += 8;
                }
            }
        } else {
            for i in 0..self.encoded_samples {
                for j in 0..self.i32_count {
                    self.len += put_varint32(&mut self.buf[self.len..], self.values[i][j])
                }
            }
        }

        // encode final quality values using RLE
        // for i := range self.quality_history {
        self.quality_history.iter_mut().for_each(|history| {
            let n_sample = len(self.quality_history[i]);
            // override final number of samples to zero
            history[n_sample - 1].samples = 0; // TODO: .last

            // otherwise, encode each value
            // for j := range self.quality_history[i] {
            history.iter().for_each(|sample| {
                self.len += put_uvarint32(&mut self.buf[self.len..], sample.value);
                self.len += put_uvarint32(&mut self.buf[self.len..], sample.samples);
            });
        });

        // reset quality history
        // for i := range self.quality_history {
        self.quality_history.iter_mut().for_each(|history| {
            // self.quality_history[i] = self.quality_history[i][:1]
            // self.quality_history[i][0].value = 0
            // self.quality_history[i][0].samples = 0
            history.clear();
            history.push(QualityHistory {
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
        // TODO determine if buf_a/buf_b can be replaced with this internal double buffering
        let active_out_buf = if !self.use_buf_a {
            self.out_buf_b
        } else {
            self.out_buf_a
        };

        // TODO inspect performance here
        active_out_buf.Reset();
        if self.encoded_samples > USE_GZIP_THRESHOLD_SAMPLES {
            // do not compress header
            active_out_buf.Write(self.buf[..actual_header_len]);

            let (gz, _) = gzip.NewWriterLevel(active_out_buf, gzip.BestCompression); // can test entropy coding by using gzip.HuffmanOnly
            if let Err(err) = gz.Write(self.buf[actual_header_len..self.len]) {
                log.Error().Err(err).Msg("could not write gz");
            }
            if let Err(err) = gz.Close() {
                log.Error().Err(err).Msg("could not close gz");
            };

            // ensure that gzip size is never greater that input for all input sizes
            if active_out_buf.Len() > self.len && self.encoded_samples == self.samples_per_message {
                log.Error()
                    .Int("gz", active_out_buf.Len())
                    .Int("original", self.len)
                    .Int("SamplesPerMessage", self.samples_per_message)
                    .Msg("gzip encoding length greater")
            }
        } else {
            active_out_buf.Write(self.buf[..self.len]);
        }

        // reset previous values
        // finalLen = self.len
        self.encoded_samples = 0;
        self.len = 0;

        // send data and swap ping-pong buffer
        if self.use_buf_a {
            self.use_buf_a = false;
            self.buf = self.buf_b;
            return Ok((active_out_buf.Bytes(), active_out_buf.Len()));
            // return Ok((self.buf_a[..finalLen], finalLen))
        }

        self.use_buf_a = true;
        self.buf = self.buf_a;
        Ok((active_out_buf.Bytes(), active_out_buf.Len()))
        // return self.buf_b[0:finalLen], finalLen, nil
    }
}
