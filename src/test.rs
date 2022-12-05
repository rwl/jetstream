use crate::decoder::Decoder;
use crate::emulator::{Emulator, ThreePhaseEmulation};
use crate::encoder::Encoder;
use crate::jetstream::DatasetWithQuality;
use lazy_static::lazy_static;
use std::collections::HashMap;

struct Test {
    sampling_rate: usize,
    count_of_variables: usize,
    samples: usize,
    samples_per_message: usize,
    quality_change: bool,
    early_encoding_stop: bool,
    use_spatial_refs: bool,
    include_neutral: bool,
    expected_size: f64, // percentage of pre-encoding size
}

impl Default for Test {
    fn default() -> Self {
        Self {
            sampling_rate: 0,
            count_of_variables: 0,
            samples: 0,
            samples_per_message: 0,
            quality_change: false,
            early_encoding_stop: false,
            use_spatial_refs: false,
            include_neutral: false,
            expected_size: 0.0,
        }
    }
}

lazy_static! {
    static ref TESTS: HashMap<String, Test> = HashMap::from([
        (
            "a10-1".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 10,
                samples_per_message: 1,
                expected_size: 53.0,
                ..Default::default()
            },
        ),
        (
            "a10-2".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 10,
                samples_per_message: 2,
                expected_size: 37.0,
                ..Default::default()
            },
        ),
        (
            "a10-2q".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 10,
                samples_per_message: 2,
                quality_change: true,
                expected_size: 37.0,
                ..Default::default()
            },
        ),
        (
            "a10-10".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 10,
                samples_per_message: 10,
                expected_size: 37.0,
                ..Default::default()
            },
        ),
        (
            "a4-2q".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 4,
                samples_per_message: 2,
                quality_change: true,
                expected_size: 37.0,
                ..Default::default()
            },
        ),
        (
            "a8-8q".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 8,
                samples_per_message: 8,
                quality_change: true,
                expected_size: 24.0,
                ..Default::default()
            },
        ),
        (
            "b4000-2".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 4000,
                samples_per_message: 2,
                expected_size: 37.0,
                ..Default::default()
            },
        ),
        (
            "b4000-80".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 4000,
                samples_per_message: 80,
                expected_size: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-60".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 4000,
                samples_per_message: 60,
                expected_size: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-800".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 800,
                samples_per_message: 800,
                expected_size: 17.0,
                ..Default::default()
            },
        ),
        (
            "b4000-4000".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 4000,
                samples_per_message: 4000,
                expected_size: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-4000s1".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 16,
                samples: 4000,
                samples_per_message: 4000,
                use_spatial_refs: false,
                expected_size: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-4000s2".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 16,
                samples: 4000,
                samples_per_message: 4000,
                use_spatial_refs: true,
                expected_size: 18.0,
                ..Default::default()
            },
        ),
        (
            "c4800-2".to_string(),
            Test {
                sampling_rate: 4800,
                count_of_variables: 8,
                samples: 4800,
                samples_per_message: 2,
                expected_size: 36.0,
                ..Default::default()
            },
        ),
        (
            "c4800-20".to_string(),
            Test {
                sampling_rate: 4800,
                count_of_variables: 8,
                samples: 4800,
                samples_per_message: 20,
                expected_size: 20.0,
                ..Default::default()
            },
        ),
        (
            "d14400-6".to_string(),
            Test {
                sampling_rate: 14400,
                count_of_variables: 8,
                samples: 14400,
                samples_per_message: 6,
                expected_size: 24.0,
                ..Default::default()
            },
        ),
        (
            "d4000-4000q".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 4000,
                samples_per_message: 4000,
                quality_change: true,
                expected_size: 17.0,
                ..Default::default()
            },
        ),
        (
            "e14400-14400".to_string(),
            Test {
                sampling_rate: 14400,
                count_of_variables: 8,
                samples: 14400,
                samples_per_message: 14400,
                expected_size: 36.0,
                ..Default::default()
            },
        ),
        (
            "e14400-14400s".to_string(),
            Test {
                sampling_rate: 14400,
                count_of_variables: 8,
                samples: 14400,
                samples_per_message: 14400,
                early_encoding_stop: true,
                expected_size: 20.0,
                ..Default::default()
            },
        ),
        (
            "e14400-14400q".to_string(),
            Test {
                sampling_rate: 14400,
                count_of_variables: 8,
                samples: 14400,
                samples_per_message: 14400,
                quality_change: true,
                expected_size: 18.0,
                ..Default::default()
            },
        ),
        (
            "f40000-40000".to_string(),
            Test {
                sampling_rate: 4000,
                count_of_variables: 8,
                samples: 40000,
                samples_per_message: 40000,
                expected_size: 17.0,
                ..Default::default()
            },
        ),
        (
            "g150000-150000".to_string(),
            Test {
                sampling_rate: 150000,
                count_of_variables: 8,
                samples: 150000,
                samples_per_message: 150000,
                expected_size: 16.0,
                ..Default::default()
            },
        ),
    ]);
}

fn create_emulator(sampling_rate: usize, phase_offset_deg: f64) -> Emulator {
    let mut emu = Emulator::new(sampling_rate, 50.03);

    emu.v = Some(ThreePhaseEmulation {
        pos_seq_mag: 400000.0 / math.Sqrt(3) * math.Sqrt(2),
        noise_max: 0.000001,
        phase_offset: phase_offset_deg * math.Pi / 180.0,

        ..Default::default()
    });
    emu.i = Some(ThreePhaseEmulation {
        pos_seq_mag: 500.0,
        phase_offset: phase_offset_deg * math.Pi / 180.0,
        harmonic_numbers: vec![5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0, 25.0],
        harmonic_mags: vec![
            0.2164, 0.1242, 0.0892, 0.0693, 0.0541, 0.0458, 0.0370, 0.0332,
        ],
        harmonic_angs: vec![171.5, 100.4, -52.4, 128.3, 80.0, 2.9, -146.8, 133.9],
        noise_max: 0.00001,

        ..Default::default()
    });

    emu
}

fn benchmark_encode_decode(b1: testing::B) {
    // let keys = Vec::with_capacity(TESTS.len());
    // for k := range tests {
    let mut keys = TESTS.keys().map(|k| k).collect::<Vec<String>>();
    // TESTS.keys().for_each(|k| {
    // 	keys = append(keys, k)
    // }
    // sort.Strings(keys)
    keys.sort();

    keys.iter().for_each(|name| {
        // for _, name := range keys {
        b1.Run(name, |b: testing::B| {
            for _i in 0..b.N {
                b.StopTimer();

                let test = TESTS.get(name).unwrap();

                // settings for IED emulator
                let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

                // initialise data structure for input data
                let mut data: Vec<DatasetWithQuality> = create_input_data(
                    &mut ied,
                    test.samples,
                    test.count_of_variables,
                    test.quality_change,
                );

                // create encoder and decoder
                let mut stream = Encoder::new(
                    ID,
                    test.count_of_variables,
                    test.sampling_rate,
                    test.samples_per_message,
                );
                let mut stream_decoder = Decoder::new(
                    ID,
                    test.count_of_variables,
                    test.sampling_rate,
                    test.samples_per_message,
                );

                b.StartTimer();

                // encode the data
                // when each message is complete, decode
                encode_and_decode(
                    nil,
                    &mut data,
                    &mut stream,
                    &mut stream_decoder,
                    test.count_of_variables,
                    test.samples_per_message,
                    test.early_encoding_stop,
                );
            }
        });
    });
}

fn benchmark_encode(b1: testing::B) {
    // keys := make([]string, 0, len(tests))
    // for k := range tests {
    // 	keys = append(keys, k)
    // }
    // sort.Strings(keys)

    let mut keys = TESTS.keys().map(|k| k).collect::<Vec<String>>();
    keys.sort();

    keys.iter().for_each(|name| {
        // for _, name := range keys {
        b1.Run(name, |b: testing::B| {
            for i in 0..b.N {
                b.StopTimer();

                let test = TESTS.get(name).unwrap();

                // settings for IED emulator
                let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

                // initialise data structure for input data
                let data: Vec<DatasetWithQuality> = create_input_data(
                    &mut ied,
                    test.samples,
                    test.count_of_variables,
                    test.quality_change,
                );

                // create encoder and decoder
                let enc = NewEncoder(
                    ID,
                    test.count_of_variables,
                    test.sampling_rate,
                    test.samples_per_message,
                );
                let dec = NewDecoder(
                    ID,
                    test.count_of_variables,
                    test.sampling_rate,
                    test.samples_per_message,
                );

                // calling b.StartTimer() often slows things down
                b.StartTimer();
                // for d := range data {
                data.iter().for_each(|d| {
                    let (buf, len, _) = enc.Encode(d);

                    if len > 0 {
                        b.StopTimer();
                        dec.DecodeToBuffer(buf, len);
                        b.StartTimer();
                    }
                });
            }
        })
    });
}

fn benchmark_decode(b1: testing::B) {
    // keys := make([]string, 0, len(tests))
    // for k := range tests {
    // 	keys = append(keys, k)
    // }
    // sort.Strings(keys)
    let mut keys = TESTS.keys().map(|k| k).collect::<Vec<String>>();
    keys.sort();

    keys.iter().for_each(|name| {
        // for _, name := range keys {
        b1.Run(name, |b: testing::B| {
            for i in 0..b.N {
                b.StopTimer();

                let test = TESTS.get(name).unwrap();

                // settings for IED emulator
                let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

                // initialise data structure for input data
                let data: Vec<DatasetWithQuality> = create_input_data(
                    &mut ied,
                    test.samples,
                    test.count_of_variables,
                    test.quality_change,
                );

                // create encoder and decoder
                let enc = NewEncoder(
                    ID,
                    test.count_of_variables,
                    test.sampling_rate,
                    test.samples_per_message,
                );
                let dec = NewDecoder(
                    ID,
                    test.count_of_variables,
                    test.sampling_rate,
                    test.samples_per_message,
                );

                // for d := range data {
                data.iter().for_each(|d| {
                    let (buf, len, _) = enc.Encode(d);

                    if len > 0 {
                        b.StartTimer();
                        dec.DecodeToBuffer(buf, len);
                        b.StopTimer();
                    }
                });
            }
        });
    });
}

fn create_input_data(
    ied: &mut Emulator,
    samples: usize,
    count_of_variables: usize,
    quality_change: bool,
) -> Vec<DatasetWithQuality> {
    let mut data: Vec<DatasetWithQuality> = vec![DatasetWithQuality; samples];
    // for i := range data {
    data.iter_mut().for_each(|d| {
        d.i32s = vec![0; count_of_variables];
        d.q = vec![0; count_of_variables];
    });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().for_each(|d| {
        // compute emulated waveform data
        ied.step();

        // calculate timestamp
        d.t = (i as u64);

        // set waveform data
        d.i32s[0] = (ied.i.A * 1000.0) as i32;
        d.i32s[1] = (ied.i.B * 1000.0) as i32;
        d.i32s[2] = (ied.i.C * 1000.0) as i32;
        d.i32s[3] = ((ied.i.A + ied.i.B + ied.i.C) * 1000.0) as i32;
        d.i32s[4] = (ied.v.A * 100.0) as i32;
        d.i32s[5] = (ied.v.B * 100.0) as i32;
        d.i32s[6] = (ied.v.C * 100.0) as i32;
        d.i32s[7] = ((ied.v.A + ied.v.B + ied.v.C) * 100.0) as i32;

        // set quality data
        d.q[0] = 0;
        d.q[1] = 0;
        d.q[2] = 0;
        d.q[3] = 0;
        d.q[4] = 0;
        d.q[5] = 0;
        d.q[6] = 0;
        d.q[7] = 0;

        if quality_change {
            if i == 2 {
                d.q[0] = 1
            } else if i == 3 {
                d.q[0] = 0x41
            }
        }
    });
    data
}

fn create_input_data_dual_ied(
    ied1: &mut Emulator,
    ied2: &mut Emulator,
    samples: usize,
    count_of_variables: usize,
    quality_change: bool,
) -> Vec<DatasetWithQuality> {
    let mut data: Vec<DatasetWithQuality> = vec![DatasetWithQuality; samples];
    data.iter_mut().for_each(|d| {
        d.i32s = vec![0; count_of_variables];
        d.q = vec![0; count_of_variables];
    });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().for_each(|d| {
        // compute emulated waveform data
        ied1.step();
        ied2.step();

        // calculate timestamp
        d.t = (i as u64);

        // set waveform data
        d.i32s[0] = (ied1.v.A * 100.0) as i32;
        d.i32s[1] = (ied1.v.B * 100.0) as i32;
        d.i32s[2] = (ied1.v.C * 100.0) as i32;
        d.i32s[3] = ((ied1.v.A + ied1.v.B + ied1.v.C) * 100.0) as i32;
        d.i32s[4] = (ied2.v.A * 100.0) as i32;
        d.i32s[5] = (ied2.v.B * 100.0) as i32;
        d.i32s[6] = (ied2.v.C * 100.0) as i32;
        d.i32s[7] = ((ied2.v.A + ied2.v.B + ied2.v.C) * 100.0) as i32;

        d.i32s[8] = (ied1.i.A * 1000.0) as i32;
        d.i32s[9] = (ied1.i.B * 1000.0) as i32;
        d.i32s[10] = (ied1.i.C * 1000.0) as i32;
        d.i32s[11] = ((ied1.i.A + ied1.i.B + ied1.i.C) * 1000.0) as i32;
        d.i32s[12] = (ied2.i.A * 1000.0) as i32;
        d.i32s[13] = (ied2.i.B * 1000.0) as i32;
        d.i32s[14] = (ied2.i.C * 1000.0) as i32;
        d.i32s[15] = ((ied2.i.A + ied2.i.B + ied2.i.C) * 1000.0) as i32;

        // set quality data
        d.q[0] = 0;
        d.q[1] = 0;
        d.q[2] = 0;
        d.q[3] = 0;
        d.q[4] = 0;
        d.q[5] = 0;
        d.q[6] = 0;
        d.q[7] = 0;
        d.q[8] = 0;
        d.q[9] = 0;
        d.q[10] = 0;
        d.q[11] = 0;
        d.q[12] = 0;
        d.q[13] = 0;
        d.q[14] = 0;
        d.q[15] = 0;

        if quality_change {
            if i == 2 {
                d.q[0] = 1;
            } else if i == 3 {
                d.q[0] = 0x41;
            }
        }
    });
    data
}

struct EncodeStats {
    samples: usize,
    messages: usize,
    total_bytes: usize,
    total_header_bytes: usize,
}

const EARLY_ENCODING_STOP_SAMPLES: usize = 100;

fn encode_and_decode(
    t: &testing::T,
    data: &mut [DatasetWithQuality],
    enc: &mut Encoder,
    dec: &mut Decoder,
    _count_of_variables: usize,
    _samples_per_message: usize,
    early_encoding_stop: bool,
) -> Result<EncodeStats, String> {
    let mut encode_stats = EncodeStats {
        samples: 0,
        messages: 0,
        total_bytes: 0,
        total_header_bytes: 0,
    };
    let mut total_samples_read = 0;

    for i in 0..data.len() {
        // data.iter_mut().for_each(|d| {
        encode_stats.samples += 1;
        let (buf, length) = enc.encode(data.get_mut(i).unwrap())?;

        // simulate encoding stopping early
        if early_encoding_stop && length == 0 && i == (EARLY_ENCODING_STOP_SAMPLES - 1) {
            let (buf, length, _) = enc.end_encode();
        }

        if length > 0 {
            // generate average stats
            encode_stats.messages += 1;
            encode_stats.total_bytes += length;
            encode_stats.total_header_bytes += 24;

            dec.decode_to_buffer(&buf, length)?;

            // compare decoded output
            if t != nil {
                for i in 0..dec.out.len() {
                    // dec.Out.iter().for_each(|out| {
                    // only check up to samples encoded
                    if earlyEncodingStop && i >= EARLY_ENCODING_STOP_SAMPLES {
                        break;
                    }

                    for j in 0..dec.i32_count {
                        if !assert.Equal(
                            t,
                            (*data)[total_samples_read + i].i32s[j],
                            dec.out[i].i32s[j],
                        ) {
                            // fmt.Println("error at", i, j)
                            t.FailNow();
                        }
                        // fmt.Println("fine at", i, j, (*data)[total_samples_read+i].int32s[j], dec.Out[i].int32s[j])
                        if !assert.Equal(t, (*data)[total_samples_read + i].q[j], dec.out[i].q[j]) {
                            // fmt.Println("Q fail:", (*data)[total_samples_read+i].Q[j], dec.Out[i].Q[j], i, j)
                            t.FailNow();
                        }
                    }
                }
            }

            total_samples_read += enc.samples_per_message;

            if earlyEncodingStop {
                return Ok(encode_stats);
            }
        }
    }

    Ok(encode_stats)
}

#[test]
pub fn test_encode_decode(t: testing::T) {
    // prepare table for presenting results
    let tab = table::Writer::new();
    tab.SetOutputMirror(os.Stdout);
    tab.SetStyle(table.StyleLight);
    tab.AppendHeader(/*table::Row{*/ [
        "samples",
        "sampling\nrate",
        "samples\nper message",
        "messages",
        "quality\nchange",
        "early\nencode stop",
        "spatial\nrefs",
        "size\n(bytes)",
        "size\n(%)",
    ]);

    // keys := make([]string, 0, len(tests))
    // for k := range tests {
    // 	keys = append(keys, k)
    // }
    // sort.Strings(keys)
    let mut keys = TESTS.keys().map(|k| k).collect::<Vec<String>>();
    keys.sort();

    // for _, name := range keys {
    keys.iter().for_each(|name| {
        t.Run(name, |t: testing::T| {
            // t.Parallel()
            let test = TESTS.get(name).unwrap();

            // settings for IED emulator
            let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);

            // initialise data structure for input data
            let mut data: Vec<DatasetWithQuality> = if test.count_of_variables == 16 {
                let mut ied2: Emulator = create_emulator(test.sampling_rate, 0.0);
                create_input_data_dual_ied(
                    &mut ied,
                    &mut ied2,
                    test.samples,
                    test.count_of_variables,
                    test.quality_change,
                )
            } else {
                create_input_data(
                    &mut ied,
                    test.samples,
                    test.count_of_variables,
                    test.quality_change,
                )
            };

            // create encoder and decoder
            let mut stream = Encoder::new(
                ID,
                test.count_of_variables,
                test.sampling_rate,
                test.samples_per_message,
            );
            let mut stream_decoder = Decoder::new(
                ID,
                test.count_of_variables,
                test.sampling_rate,
                test.samples_per_message,
            );

            if test.use_spatial_refs {
                stream.set_spatial_refs(
                    test.count_of_variables,
                    test.count_of_variables / 8,
                    test.count_of_variables / 8,
                    true,
                ); // TODO test include_neutral
                stream_decoder.set_spatial_refs(
                    test.count_of_variables,
                    test.count_of_variables / 8,
                    test.count_of_variables / 8,
                    true,
                ); // TODO test include_neutral
            }

            // encode the data
            // when each message is complete, decode
            let (encode_stats, _) = encode_and_decode(
                t,
                &mut data,
                &mut stream,
                &mut stream_decoder,
                test.count_of_variables,
                test.samples_per_message,
                test.early_encoding_stop,
            );

            let theory_bytes_per_message = if test.early_encoding_stop {
                test.count_of_variables * encode_stats.samples * 16
            } else {
                test.count_of_variables * test.samples_per_message * 16
            };
            let mean_bytes_per_message =
                (encode_stats.totalBytes as f64) / (encode_stats.messages as f64); // includes header overhead
            let percent =
                100.0 * (mean_bytes_per_message as f64) / (theory_bytes_per_message as f64);
            // meanBytesWithoutHeader := float64(encode_stats.total_bytes-encode_stats.total_header_bytes) / float64(encode_stats.iterations)

            assert.LessOrEqual(t, percent, tests[name].expectedSize);

            tab.AppendRow([
                encode_stats.samples,
                tests[name].samplingRate,
                tests[name].samplesPerMessage,
                encode_stats.messages,
                tests[name].qualityChange,
                tests[name].earlyEncodingStop,
                tests[name].useSpatialRefs,
                format!("{:.1}", mean_bytes_per_message),
                format!("{:.1}", percent),
            ]);
            // tab.AppendSeparator()
        });
    });

    // show table of results
    tab.Render();
    // tab.RenderCSV()
}

#[test]
fn test_wrong_id(t: testing::T) {
    t.Run("wrong ID", |t: testing::T| {
        if let Some(test) = TESTS.get("a10-1") {
            // test := tests["a10-1"]

            // settings for IED emulator
            let mut ied: Emulator = create_emulator(test.sampling_rate, 0.0);
            let wrong_id: uuid::Uuid = uuid::Uuid::new_v4();

            // initialise data structure for input data
            let mut data: Vec<DatasetWithQuality> = create_input_data(
                &mut ied,
                test.samples,
                test.count_of_variables,
                test.quality_change,
            );

            // create encoder and decoder
            let mut stream = Encoder::new(
                ID,
                test.count_of_variables,
                test.sampling_rate,
                test.samples_per_message,
            );
            let mut stream_decoder = Decoder::new(
                wrong_id,
                test.count_of_variables,
                test.sampling_rate,
                test.samples_per_message,
            );

            // encode the data
            // when each message is complete, decode
            let err = encode_and_decode(
                t,
                &mut data,
                &mut stream,
                &mut stream_decoder,
                test.count_of_variables,
                test.samples_per_message,
                test.early_encoding_stop,
            );
            assert.Equal(t, err.Error(), "IDs did not match");
        } else {
            t.Log("Test data missing");
            t.Fail();
        }
    });
}
