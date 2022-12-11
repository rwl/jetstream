use crate::decoder::Decoder;
use crate::emulator::{Emulator, ThreePhaseEmulation};
use crate::encoder::Encoder;
use crate::jetstream::DatasetWithQuality;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Default)]
pub struct TestCase {
    pub sampling_rate: usize,
    pub count_of_variables: usize,
    pub samples: usize,
    pub samples_per_message: usize,
    pub quality_change: bool,
    pub early_encoding_stop: bool,
    pub use_spatial_refs: bool,
    pub include_neutral: bool,
    pub expected_size: f64, // percentage of pre-encoding size
}

lazy_static! {
    // static ref ID: uuid::Uuid = uuid::Uuid::new_v4();
    pub static ref TESTS: HashMap<String, TestCase> = HashMap::from([
        (
            "a10-1".to_string(),
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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
            TestCase {
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

pub fn create_emulator(sampling_rate: usize, phase_offset_deg: f64) -> Emulator {
    let mut emu = Emulator::new(sampling_rate, 50.03);

    emu.v = Some(ThreePhaseEmulation {
        pos_seq_mag: 400000.0 / f64::sqrt(3.0) * f64::sqrt(2.0),
        noise_max: 0.000001,
        phase_offset: phase_offset_deg * PI / 180.0,

        ..Default::default()
    });
    emu.i = Some(ThreePhaseEmulation {
        pos_seq_mag: 500.0,
        phase_offset: phase_offset_deg * PI / 180.0,
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

#[derive(Debug)]
pub struct EncodeStats {
    pub samples: usize,
    pub messages: usize,
    pub total_bytes: usize,
    pub total_header_bytes: usize,
}

const EARLY_ENCODING_STOP_SAMPLES: usize = 100;

pub fn encode_and_decode(
    compare: bool,
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
        let (buf, length) =
            if early_encoding_stop && length != 0 && i == (EARLY_ENCODING_STOP_SAMPLES - 1) {
                enc.end_encode()?
            } else {
                (buf, length)
            };

        if length > 0 {
            // generate average stats
            encode_stats.messages += 1;
            encode_stats.total_bytes += length;
            encode_stats.total_header_bytes += 24;

            dec.decode_to_buffer(&buf, length)?;

            // compare decoded output
            if compare {
                for i in 0..dec.out.len() {
                    // dec.Out.iter().for_each(|out| {
                    // only check up to samples encoded
                    if early_encoding_stop && i >= EARLY_ENCODING_STOP_SAMPLES {
                        break;
                    }

                    for j in 0..dec.i32_count {
                        assert_eq!(
                            (*data)[total_samples_read + i].i32s[j],
                            dec.out[i].i32s[j],
                            "error at {},{}",
                            i,
                            j
                        );

                        // println!("fine at {},{} = {}", i, j, (*data)[total_samples_read+i].i32s[j], dec.out[i].i32s[j]);
                        assert_eq!(
                            (*data)[total_samples_read + i].q[j],
                            dec.out[i].q[j],
                            "Q fail: {} != {} - ({},{})",
                            (*data)[total_samples_read + i].q[j],
                            dec.out[i].q[j],
                            i,
                            j
                        );
                    }
                }
            }

            total_samples_read += enc.samples_per_message;

            if early_encoding_stop {
                return Ok(encode_stats);
            }
        }
    }

    Ok(encode_stats)
}
