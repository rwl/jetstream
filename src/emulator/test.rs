use crate::emulator::{Emulator, SagEmulation, TemperatureEmulation, ThreePhaseEmulation};
use std::collections::HashMap;
use std::f64::consts::PI;

// benchmark emulator performance
fn benchmark_emulator(b: testing::B) {
    let mut emu = create_emulator_for_benchmark(4000, 0.0);

    for _ in 0..b.N {
        for _ in 0..4000 {
            emu.step();
        }
    }
}

fn create_emulator_for_benchmark(sampling_rate: usize, phase_offset_deg: f64) -> Emulator {
    let mut emu = Emulator::new(sampling_rate, 50.0);

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
        noise_max: 0.000001,
        ..Default::default()
    });

    emu
}

fn create_emulator(sampling_rate: usize, phase_offset_deg: f64) -> Emulator {
    let mut emu = Emulator::new(sampling_rate, 50.0);

    emu.v = Some(ThreePhaseEmulation {
        pos_seq_mag: 400000.0 / f64::sqrt(3.0) * f64::sqrt(2.0),
        noise_max: 0.000001,
        phase_offset: phase_offset_deg * PI / 180.0,
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
        noise_max: 0.000001,
        ..Default::default()
    });
    emu.t = Some(TemperatureEmulation {
        mean_temperature: 30.0,
        noise_max: 0.01,
        instantaneous_anomaly_magnitude: 30.0,
        instantaneous_anomaly_probability: 0.01,
        ..Default::default()
    });
    emu
}

fn floating_point_equal(expected: f64, actual: f64, threshold: f64) -> bool {
    let abs_diff = (expected - actual).abs();
    abs_diff < threshold
}

fn mean(values: &[f64]) -> f64 {
    let mut sum: f64 = 0.0;
    values.iter().for_each(|value| sum += value);
    sum / (values.len() as f64)
}

#[test]
fn test_temperature_emulation_anomalies__no_anomalies(t: testing::T) {
    let mut emulator = create_emulator(14400, 0.0);

    emulator
        .t
        .as_mut()
        .unwrap()
        .instantaneous_anomaly_probability = 0.0;
    let mut step = 0;
    let mut results: Vec<bool> = vec![];
    while step < 1e4 {
        emulator.step();
        results.push(emulator.t.isInstantaneousAnomaly);
        step += 1;
    }
    assert.NotContains(t, results, true);
}

#[test]
fn test_temperature_emulation_anomalies__anomalies(t: testing::T) {
    let mut emulator = create_emulator(14400, 0.0);

    emulator
        .t
        .as_mut()
        .unwrap()
        .instantaneous_anomaly_probability = 0.5;
    let mut step = 0;
    let mut results: Vec<bool> = vec![];
    let mut normal_values: Vec<f64> = vec![];
    let mut anomaly_values: Vec<f64> = vec![];
    while step < 1e4 {
        emulator.step();
        results.push(emulator.t.unwrap().is_instantaneous_anomaly);

        if emulator.t.unwrap().is_instantaneous_anomaly == true {
            anomaly_values.push(emulator.t.unwrap().t);
        } else {
            normal_values.push(emulator.t.unwrap().t);
        }
        step += 1;
    }
    assert.Contains(t, results, true);

    let fraction_anomalies = (anomaly_values.len() as f64) / (step as f64);
    assert.True(t, floating_point_equal(0.5, fraction_anomalies, 0.1));

    assert.True(t, mean(&anomaly_values) > mean(&normal_values));
}

#[test]
fn test_temperature_emulation_anomalies__rising_trend(t: testing::T) {
    let mut emulator = create_emulator(14400, 0.0);
    emulator.t.as_mut().unwrap().is_trend_anomaly = true;
    emulator.t.as_mut().unwrap().trend_anomaly_magnitude = 30.0;
    emulator.t.as_mut().unwrap().trend_anomaly_duration = 10;
    emulator.t.as_mut().unwrap().is_rising_trend_anomaly = true;

    let mut step = 0;
    let mut results: Vec<f64> = vec![];
    while step < emulator.t.unwrap().trend_anomaly_duration * emulator.sampling_rate {
        emulator.step();
        results.push(emulator.t.unwrap().t);
        step += 1;

        if step < emulator.sampling_rate {
            assert.NotEqual(t, 0, emulator.t.unwrap().trend_anomaly_index);
        }
    }

    assert.True(t, mean(&results) > emulator.t.MeanTemperature);
}

#[test]
fn test_temperature_emulation_anomalies__decreasing_trend(t: testing::T) {
    let mut emulator = create_emulator(1, 0.0);
    emulator.t.as_mut().unwrap().is_trend_anomaly = true;
    emulator.t.as_mut().unwrap().trend_anomaly_magnitude = 30.0;
    emulator.t.as_mut().unwrap().trend_anomaly_duration = 10;
    emulator.t.as_mut().unwrap().is_rising_trend_anomaly = false;
    let mut step = 0;
    let mut results: Vec<f64> = vec![];
    while step < 10 {
        emulator.step();
        results.push(emulator.t.unwrap().t);
        step += 1;
    }

    assert.True(t, mean(&results) < emulator.t.unwrap().mean_temperature);
}

#[test]
fn test_sag_emulation(t: testing::T) {
    let mut emulator = create_emulator(14400, 0.0);
    emulator.sag = Some(SagEmulation {
        mean_calculated_temperature: 30.0,
        mean_strain: 100.0,
        mean_sag: 0.5,
        ..Default::default()
    });

    let mut step = 0;
    let mut results = HashMap::<String, Vec<f64>>::new();
    results["CalculatedTemperature"] = vec![];
    results["TotalStrain"] = vec![];
    results["Sag"] = vec![];

    while step < 1e4 {
        emulator.step();
        results["CalculatedTemperature"].push(emulator.sag.CalculatedTemperature);
        results["TotalStrain"].push(emulator.sag.CalculatedTemperature);
        results["Sag"].push(emulator.sag.CalculatedTemperature);
        step += 1;
    }

    // for _, field := range []string{"CalculatedTemperature", "TotalStrain", "Sag"} {
    // 	assert.IsType(t, []float64{}, results[field])
    // }
}
