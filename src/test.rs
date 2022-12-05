use crate::decoder::Decoder;
use crate::emulator::{Emulator, ThreePhaseEmulation};
use crate::encoder::Encoder;
use crate::jetstream::DatasetWithQuality;
use lazy_static::lazy_static;
use std::collections::HashMap;

struct Test {
    samplingRate: usize,
    countOfVariables: usize,
    samples: usize,
    samplesPerMessage: usize,
    qualityChange: bool,
    earlyEncodingStop: bool,
    useSpatialRefs: bool,
    includeNeutral: bool,
    expectedSize: f64, // percentage of pre-encoding size
}

impl Default for Test {
    fn default() -> Self {
        Self {
            samplingRate: 0,
            countOfVariables: 0,
            samples: 0,
            samplesPerMessage: 0,
            qualityChange: false,
            earlyEncodingStop: false,
            useSpatialRefs: false,
            includeNeutral: false,
            expectedSize: 0.0,
        }
    }
}

lazy_static! {
    static ref TESTS: HashMap<String, Test> = HashMap::from([
        (
            "a10-1".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 10,
                samplesPerMessage: 1,
                expectedSize: 53.0,
                ..Default::default()
            },
        ),
        (
            "a10-2".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 10,
                samplesPerMessage: 2,
                expectedSize: 37.0,
                ..Default::default()
            },
        ),
        (
            "a10-2q".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 10,
                samplesPerMessage: 2,
                qualityChange: true,
                expectedSize: 37.0,
                ..Default::default()
            },
        ),
        (
            "a10-10".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 10,
                samplesPerMessage: 10,
                expectedSize: 37.0,
                ..Default::default()
            },
        ),
        (
            "a4-2q".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 4,
                samplesPerMessage: 2,
                qualityChange: true,
                expectedSize: 37.0,
                ..Default::default()
            },
        ),
        (
            "a8-8q".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 8,
                samplesPerMessage: 8,
                qualityChange: true,
                expectedSize: 24.0,
                ..Default::default()
            },
        ),
        (
            "b4000-2".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 4000,
                samplesPerMessage: 2,
                expectedSize: 37.0,
                ..Default::default()
            },
        ),
        (
            "b4000-80".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 4000,
                samplesPerMessage: 80,
                expectedSize: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-60".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 4000,
                samplesPerMessage: 60,
                expectedSize: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-800".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 800,
                samplesPerMessage: 800,
                expectedSize: 17.0,
                ..Default::default()
            },
        ),
        (
            "b4000-4000".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 4000,
                samplesPerMessage: 4000,
                expectedSize: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-4000s1".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 16,
                samples: 4000,
                samplesPerMessage: 4000,
                useSpatialRefs: false,
                expectedSize: 18.0,
                ..Default::default()
            },
        ),
        (
            "b4000-4000s2".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 16,
                samples: 4000,
                samplesPerMessage: 4000,
                useSpatialRefs: true,
                expectedSize: 18.0,
                ..Default::default()
            },
        ),
        (
            "c4800-2".to_string(),
            Test {
                samplingRate: 4800,
                countOfVariables: 8,
                samples: 4800,
                samplesPerMessage: 2,
                expectedSize: 36.0,
                ..Default::default()
            },
        ),
        (
            "c4800-20".to_string(),
            Test {
                samplingRate: 4800,
                countOfVariables: 8,
                samples: 4800,
                samplesPerMessage: 20,
                expectedSize: 20.0,
                ..Default::default()
            },
        ),
        (
            "d14400-6".to_string(),
            Test {
                samplingRate: 14400,
                countOfVariables: 8,
                samples: 14400,
                samplesPerMessage: 6,
                expectedSize: 24.0,
                ..Default::default()
            },
        ),
        (
            "d4000-4000q".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 4000,
                samplesPerMessage: 4000,
                qualityChange: true,
                expectedSize: 17.0,
                ..Default::default()
            },
        ),
        (
            "e14400-14400".to_string(),
            Test {
                samplingRate: 14400,
                countOfVariables: 8,
                samples: 14400,
                samplesPerMessage: 14400,
                expectedSize: 36.0,
                ..Default::default()
            },
        ),
        (
            "e14400-14400s".to_string(),
            Test {
                samplingRate: 14400,
                countOfVariables: 8,
                samples: 14400,
                samplesPerMessage: 14400,
                earlyEncodingStop: true,
                expectedSize: 20.0,
                ..Default::default()
            },
        ),
        (
            "e14400-14400q".to_string(),
            Test {
                samplingRate: 14400,
                countOfVariables: 8,
                samples: 14400,
                samplesPerMessage: 14400,
                qualityChange: true,
                expectedSize: 18.0,
                ..Default::default()
            },
        ),
        (
            "f40000-40000".to_string(),
            Test {
                samplingRate: 4000,
                countOfVariables: 8,
                samples: 40000,
                samplesPerMessage: 40000,
                expectedSize: 17.0,
                ..Default::default()
            },
        ),
        (
            "g150000-150000".to_string(),
            Test {
                samplingRate: 150000,
                countOfVariables: 8,
                samples: 150000,
                samplesPerMessage: 150000,
                expectedSize: 16.0,
                ..Default::default()
            },
        ),
    ]);
}

fn createEmulator(samplingRate: usize, phaseOffsetDeg: f64) -> Emulator {
    let mut emu = Emulator::new(samplingRate, 50.03);

    emu.V = Some(ThreePhaseEmulation {
        PosSeqMag: 400000.0 / math.Sqrt(3) * math.Sqrt(2),
        NoiseMax: 0.000001,
        PhaseOffset: phaseOffsetDeg * math.Pi / 180.0,

        ..Default::default()
    });
    emu.I = Some(ThreePhaseEmulation {
        PosSeqMag: 500.0,
        PhaseOffset: phaseOffsetDeg * math.Pi / 180.0,
        HarmonicNumbers: vec![5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0, 25.0],
        HarmonicMags: vec![
            0.2164, 0.1242, 0.0892, 0.0693, 0.0541, 0.0458, 0.0370, 0.0332,
        ],
        HarmonicAngs: vec![171.5, 100.4, -52.4, 128.3, 80.0, 2.9, -146.8, 133.9],
        NoiseMax: 0.00001,

        ..Default::default()
    });

    emu
}

fn BenchmarkEncodeDecode(b1: testing::B) {
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
            for i in 0..b.N {
                b.StopTimer();

                let test = TESTS.get(name).unwrap();

                // settings for IED emulator
                let ied: Emulator = createEmulator(test.samplingRate, 0.0);

                // initialise data structure for input data
                let data: Vec<DatasetWithQuality> =
                    createInputData(ied, test.samples, test.countOfVariables, test.qualityChange);

                // create encoder and decoder
                let stream = Encoder::new(
                    ID,
                    test.countOfVariables,
                    test.samplingRate,
                    test.samplesPerMessage,
                );
                let streamDecoder = Decoder::new(
                    ID,
                    test.countOfVariables,
                    test.samplingRate,
                    test.samplesPerMessage,
                );

                b.StartTimer();

                // encode the data
                // when each message is complete, decode
                encodeAndDecode(
                    nil,
                    &data,
                    stream,
                    streamDecoder,
                    test.countOfVariables,
                    test.samplesPerMessage,
                    test.earlyEncodingStop,
                );
            }
        });
    });
}

fn BenchmarkEncode(b1: testing::B) {
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
                let ied: Emulator = createEmulator(test.samplingRate, 0.0);

                // initialise data structure for input data
                let data: Vec<DatasetWithQuality> =
                    createInputData(ied, test.samples, test.countOfVariables, test.qualityChange);

                // create encoder and decoder
                let enc = NewEncoder(
                    ID,
                    test.countOfVariables,
                    test.samplingRate,
                    test.samplesPerMessage,
                );
                let dec = NewDecoder(
                    ID,
                    test.countOfVariables,
                    test.samplingRate,
                    test.samplesPerMessage,
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

fn BenchmarkDecode(b1: testing::B) {
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
                let ied: Emulator = createEmulator(test.samplingRate, 0.0);

                // initialise data structure for input data
                let data: Vec<DatasetWithQuality> =
                    createInputData(ied, test.samples, test.countOfVariables, test.qualityChange);

                // create encoder and decoder
                let enc = NewEncoder(
                    ID,
                    test.countOfVariables,
                    test.samplingRate,
                    test.samplesPerMessage,
                );
                let dec = NewDecoder(
                    ID,
                    test.countOfVariables,
                    test.samplingRate,
                    test.samplesPerMessage,
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

fn createInputData(
    ied: &mut Emulator,
    samples: usize,
    countOfVariables: usize,
    qualityChange: bool,
) -> Vec<DatasetWithQuality> {
    let mut data: Vec<DatasetWithQuality> = vec![DatasetWithQuality; samples];
    // for i := range data {
    data.iter_mut().for_each(|d| {
        d.Int32s = vec![0; countOfVariables];
        d.Q = vec![0; countOfVariables];
    });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().for_each(|d| {
        // compute emulated waveform data
        ied.Step();

        // calculate timestamp
        d.T = (i as u64);

        // set waveform data
        d.Int32s[0] = (ied.I.A * 1000.0) as i32;
        d.Int32s[1] = (ied.I.B * 1000.0) as i32;
        d.Int32s[2] = (ied.I.C * 1000.0) as i32;
        d.Int32s[3] = ((ied.I.A + ied.I.B + ied.I.C) * 1000.0) as i32;
        d.Int32s[4] = (ied.V.A * 100.0) as i32;
        d.Int32s[5] = (ied.V.B * 100.0) as i32;
        d.Int32s[6] = (ied.V.C * 100.0) as i32;
        d.Int32s[7] = ((ied.V.A + ied.V.B + ied.V.C) * 100.0) as i32;

        // set quality data
        d.Q[0] = 0;
        d.Q[1] = 0;
        d.Q[2] = 0;
        d.Q[3] = 0;
        d.Q[4] = 0;
        d.Q[5] = 0;
        d.Q[6] = 0;
        d.Q[7] = 0;

        if qualityChange {
            if i == 2 {
                d.Q[0] = 1
            } else if i == 3 {
                d.Q[0] = 0x41
            }
        }
    });
    data
}

fn createInputDataDualIED(
    ied1: &mut Emulator,
    ied2: &Emulator,
    samples: usize,
    countOfVariables: usize,
    qualityChange: bool,
) -> Vec<DatasetWithQuality> {
    let mut data: Vec<DatasetWithQuality> = vec![DatasetWithQuality; samples];
    data.iter_mut().for_each(|d| {
        d.Int32s = vec![0; countOfVariables];
        d.Q = vec![0; countOfVariables];
    });

    // generate data using IED emulator
    // the timestamp is a simple integer counter, starting from 0
    // for i := range data {
    data.iter_mut().for_each(|d| {
        // compute emulated waveform data
        ied1.Step();
        ied2.Step();

        // calculate timestamp
        d.T = (i as u64);

        // set waveform data
        d.Int32s[0] = (ied1.V.A * 100.0) as i32;
        d.Int32s[1] = (ied1.V.B * 100.0) as i32;
        d.Int32s[2] = (ied1.V.C * 100.0) as i32;
        d.Int32s[3] = ((ied1.V.A + ied1.V.B + ied1.V.C) * 100.0) as i32;
        d.Int32s[4] = (ied2.V.A * 100.0) as i32;
        d.Int32s[5] = (ied2.V.B * 100.0) as i32;
        d.Int32s[6] = (ied2.V.C * 100.0) as i32;
        d.Int32s[7] = ((ied2.V.A + ied2.V.B + ied2.V.C) * 100.0) as i32;

        d.Int32s[8] = (ied1.I.A * 1000.0) as i32;
        d.Int32s[9] = (ied1.I.B * 1000.0) as i32;
        d.Int32s[10] = (ied1.I.C * 1000.0) as i32;
        d.Int32s[11] = ((ied1.I.A + ied1.I.B + ied1.I.C) * 1000.0) as i32;
        d.Int32s[12] = (ied2.I.A * 1000.0) as i32;
        d.Int32s[13] = (ied2.I.B * 1000.0) as i32;
        d.Int32s[14] = (ied2.I.C * 1000.0) as i32;
        d.Int32s[15] = ((ied2.I.A + ied2.I.B + ied2.I.C) * 1000.0) as i32;

        // set quality data
        d.Q[0] = 0;
        d.Q[1] = 0;
        d.Q[2] = 0;
        d.Q[3] = 0;
        d.Q[4] = 0;
        d.Q[5] = 0;
        d.Q[6] = 0;
        d.Q[7] = 0;
        d.Q[8] = 0;
        d.Q[9] = 0;
        d.Q[10] = 0;
        d.Q[11] = 0;
        d.Q[12] = 0;
        d.Q[13] = 0;
        d.Q[14] = 0;
        d.Q[15] = 0;

        if qualityChange {
            if i == 2 {
                d.Q[0] = 1;
            } else if i == 3 {
                d.Q[0] = 0x41;
            }
        }
    });
    data
}

struct encodeStats {
    samples: usize,
    messages: usize,
    totalBytes: usize,
    totalHeaderBytes: usize,
}

const earlyEncodingStopSamples: usize = 100;

fn encodeAndDecode(
    t: &testing::T,
    data: &mut [DatasetWithQuality],
    enc: &mut Encoder,
    dec: &mut Decoder,
    countOfVariables: usize,
    samplesPerMessage: usize,
    earlyEncodingStop: bool,
) -> Result<encodeStats, String> {
    let mut encodeStats = encodeStats {
        samples: 0,
        messages: 0,
        totalBytes: 0,
        totalHeaderBytes: 0,
    };
    let mut totalSamplesRead = 0;

    for i in 0..data.len() {
        // data.iter_mut().for_each(|d| {
        encodeStats.samples += 1;
        let (buf, length) = enc.Encode(data[i])?;

        // simulate encoding stopping early
        if earlyEncodingStop && length == 0 && i == (earlyEncodingStopSamples - 1) {
            let (buf, length, _) = enc.EndEncode();
        }

        if length > 0 {
            // generate average stats
            encodeStats.messages += 1;
            encodeStats.totalBytes += length;
            encodeStats.totalHeaderBytes += 24;

            dec.DecodeToBuffer(buf, length)?;

            // compare decoded output
            if t != nil {
                for i in 0..dec.Out.len() {
                    // dec.Out.iter().for_each(|out| {
                    // only check up to samples encoded
                    if earlyEncodingStop && i >= earlyEncodingStopSamples {
                        break;
                    }

                    for j in 0..dec.Int32Count {
                        if !assert.Equal(
                            t,
                            (*data)[totalSamplesRead + i].Int32s[j],
                            dec.Out[i].Int32s[j],
                        ) {
                            // fmt.Println("error at", i, j)
                            t.FailNow();
                        }
                        // fmt.Println("fine at", i, j, (*data)[totalSamplesRead+i].Int32s[j], dec.Out[i].Int32s[j])
                        if !assert.Equal(t, (*data)[totalSamplesRead + i].Q[j], dec.Out[i].Q[j]) {
                            // fmt.Println("Q fail:", (*data)[totalSamplesRead+i].Q[j], dec.Out[i].Q[j], i, j)
                            t.FailNow();
                        }
                    }
                }
            }

            totalSamplesRead += enc.SamplesPerMessage;

            if earlyEncodingStop {
                return Ok(encodeStats);
            }
        }
    }

    Ok(encodeStats)
}

pub fn TestEncodeDecode(t: testing::T) {
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
            let ied: Emulator = createEmulator(test.samplingRate, 0.0);

            // initialise data structure for input data
            let data: Vec<DatasetWithQuality> = if test.countOfVariables == 16 {
                let ied2: Emulator = createEmulator(test.samplingRate, 0.0);
                createInputDataDualIED(
                    ied,
                    ied2,
                    test.samples,
                    test.countOfVariables,
                    test.qualityChange,
                )
            } else {
                createInputData(ied, test.samples, test.countOfVariables, test.qualityChange)
            };

            // create encoder and decoder
            let mut stream = Encoder::new(
                ID,
                test.countOfVariables,
                test.samplingRate,
                test.samplesPerMessage,
            );
            let mut streamDecoder = Decoder::new(
                ID,
                test.countOfVariables,
                test.samplingRate,
                test.samplesPerMessage,
            );

            if test.useSpatialRefs {
                stream.SetSpatialRefs(
                    test.countOfVariables,
                    test.countOfVariables / 8,
                    test.countOfVariables / 8,
                    true,
                ); // TODO test includeNeutral
                streamDecoder.SetSpatialRefs(
                    test.countOfVariables,
                    test.countOfVariables / 8,
                    test.countOfVariables / 8,
                    true,
                ); // TODO test includeNeutral
            }

            // encode the data
            // when each message is complete, decode
            let (encodeStats, _) = encodeAndDecode(
                t,
                &data,
                stream,
                streamDecoder,
                test.countOfVariables,
                test.samplesPerMessage,
                test.earlyEncodingStop,
            );

            let theoryBytesPerMessage = if test.earlyEncodingStop {
                test.countOfVariables * encodeStats.samples * 16
            } else {
                test.countOfVariables * test.samplesPerMessage * 16
            };
            let meanBytesPerMessage =
                (encodeStats.totalBytes as f64) / (encodeStats.messages as f64); // includes header overhead
            let percent = 100.0 * (meanBytesPerMessage as f64) / (theoryBytesPerMessage as f64);
            // meanBytesWithoutHeader := float64(encodeStats.totalBytes-encodeStats.totalHeaderBytes) / float64(encodeStats.iterations)

            assert.LessOrEqual(t, percent, tests[name].expectedSize);

            tab.AppendRow([
                encodeStats.samples,
                tests[name].samplingRate,
                tests[name].samplesPerMessage,
                encodeStats.messages,
                tests[name].qualityChange,
                tests[name].earlyEncodingStop,
                tests[name].useSpatialRefs,
                format!("{:.1}", meanBytesPerMessage),
                format!("{:.1}", percent),
            ]);
            // tab.AppendSeparator()
        });
    });

    // show table of results
    tab.Render();
    // tab.RenderCSV()
}

fn TestWrongID(t: testing::T) {
    t.Run("wrong ID", |t: testing::T| {
        if let Some(test) = TESTS.get("a10-1") {
            // test := tests["a10-1"]

            // settings for IED emulator
            let ied: Emulator = createEmulator(test.samplingRate, 0.0);
            let wrongID: uuid::Uuid = uuid::Uuid::nil(); // FIXME

            // initialise data structure for input data
            let data: Vec<DatasetWithQuality> =
                createInputData(ied, test.samples, test.countOfVariables, test.qualityChange);

            // create encoder and decoder
            let stream = Encoder::new(
                ID,
                test.countOfVariables,
                test.samplingRate,
                test.samplesPerMessage,
            );
            let streamDecoder = Decoder::new(
                wrongID,
                test.countOfVariables,
                test.samplingRate,
                test.samplesPerMessage,
            );

            // encode the data
            // when each message is complete, decode
            let err = encodeAndDecode(
                t,
                &data,
                stream,
                streamDecoder,
                test.countOfVariables,
                test.samplesPerMessage,
                test.earlyEncodingStop,
            );
            assert.Equal(t, err.Error(), "IDs did not match");
        } else {
            t.Log("Test data missing");
            t.Fail();
        }
    });
}
