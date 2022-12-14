use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jetstream::emulator::{Emulator, ThreePhaseEmulation};
use std::f64::consts::PI;

pub fn emulator_benchmark(c: &mut Criterion) {
    const SAMPLING_RATE: usize = 4000;
    const PHASE_OFFSET_DEG: f64 = 0.0;

    let mut emu = Emulator::new(SAMPLING_RATE, 50.0);
    emu.v = Some(ThreePhaseEmulation {
        pos_seq_mag: 400000.0 / f64::sqrt(3.0) * f64::sqrt(2.0),
        noise_max: 0.000001,
        phase_offset: PHASE_OFFSET_DEG * PI / 180.0,
        ..Default::default()
    });
    emu.i = Some(ThreePhaseEmulation {
        pos_seq_mag: 500.0,
        phase_offset: PHASE_OFFSET_DEG * PI / 180.0,
        harmonic_numbers: vec![5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0, 25.0],
        harmonic_mags: vec![
            0.2164, 0.1242, 0.0892, 0.0693, 0.0541, 0.0458, 0.0370, 0.0332,
        ],
        harmonic_angs: vec![171.5, 100.4, -52.4, 128.3, 80.0, 2.9, -146.8, 133.9],
        noise_max: 0.000001,
        ..Default::default()
    });

    c.bench_function("step", |b| {
        b.iter(|| {
            for _ in 0..SAMPLING_RATE {
                black_box(&mut emu).step();
            }
        })
    });
}

criterion_group!(benches, emulator_benchmark);
criterion_main!(benches);
