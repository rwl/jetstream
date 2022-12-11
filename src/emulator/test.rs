#[cfg(test)]
use crate::emulator::SagEmulation;
use crate::emulator::{Emulator, TemperatureEmulation, ThreePhaseEmulation};
#[cfg(test)]
use std::collections::HashMap;
use std::f64::consts::PI;

#[cfg(test)]
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
        phase_offset: phase_offset_deg * PI / 180.0,
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

#[cfg(test)]
fn floating_point_equal(expected: f64, actual: f64, threshold: f64) -> bool {
    let abs_diff = (expected - actual).abs();
    abs_diff < threshold
}

#[cfg(test)]
fn mean(values: &[f64]) -> f64 {
    let mut sum: f64 = 0.0;
    values.iter().for_each(|value| sum += value);
    sum / (values.len() as f64)
}

#[test]
fn test_temperature_emulation_no_anomalies() {
    let mut emulator = create_emulator(14400, 0.0);

    emulator
        .t
        .as_mut()
        .unwrap()
        .instantaneous_anomaly_probability = 0.0;
    let mut step = 0;
    let mut results: Vec<bool> = vec![];
    while step < 10_000 {
        emulator.step();
        results.push(emulator.t.as_ref().unwrap().is_instantaneous_anomaly);
        step += 1;
    }
    assert!(!results.contains(&true));
}

#[test]
fn test_temperature_emulation_anomalies() {
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
    while step < 10_000 {
        emulator.step();

        let t = emulator.t.as_ref().unwrap();
        results.push(t.is_instantaneous_anomaly);

        if t.is_instantaneous_anomaly == true {
            anomaly_values.push(t.t);
        } else {
            normal_values.push(t.t);
        }
        step += 1;
    }
    assert!(results.contains(&true));

    let fraction_anomalies = (anomaly_values.len() as f64) / (step as f64);
    assert!(floating_point_equal(0.5, fraction_anomalies, 0.1));

    assert!(mean(&anomaly_values) > mean(&normal_values));
}

#[test]
fn test_temperature_emulation_rising_trend() {
    let mut emulator = create_emulator(14400, 0.0);
    let t = emulator.t.as_mut().unwrap();
    t.is_trend_anomaly = true;
    t.trend_anomaly_magnitude = 30.0;
    t.trend_anomaly_duration = 10;
    t.is_rising_trend_anomaly = true;

    let mut step = 0;
    let mut results: Vec<f64> = vec![];
    while step < emulator.t.as_ref().unwrap().trend_anomaly_duration * emulator.sampling_rate {
        emulator.step();
        results.push(emulator.t.as_ref().unwrap().t);
        step += 1;

        if step < emulator.sampling_rate {
            assert_ne!(0, emulator.t.as_ref().unwrap().trend_anomaly_index);
        }
    }

    assert!(mean(&results) > emulator.t.as_ref().unwrap().mean_temperature);
}

#[test]
fn test_temperature_emulation_decreasing_trend() {
    let mut emulator = create_emulator(1, 0.0);
    emulator.t.as_mut().unwrap().is_trend_anomaly = true;
    emulator.t.as_mut().unwrap().trend_anomaly_magnitude = 30.0;
    emulator.t.as_mut().unwrap().trend_anomaly_duration = 10;
    emulator.t.as_mut().unwrap().is_rising_trend_anomaly = false;
    let mut step = 0;
    let mut results: Vec<f64> = vec![];
    while step < 10 {
        emulator.step();
        results.push(emulator.t.as_ref().unwrap().t);
        step += 1;
    }

    assert!(mean(&results) < emulator.t.as_ref().unwrap().mean_temperature);
}

#[test]
fn test_sag_emulation() {
    let mut emulator = create_emulator(14400, 0.0);
    emulator.sag = Some(SagEmulation {
        mean_calculated_temperature: 30.0,
        mean_strain: 100.0,
        mean_sag: 0.5,
        ..Default::default()
    });

    let mut step = 0;
    let mut results = HashMap::<String, Vec<f64>>::new();
    results.insert("CalculatedTemperature".to_string(), vec![]);
    results.insert("TotalStrain".to_string(), vec![]);
    results.insert("Sag".to_string(), vec![]);

    while step < 10_000 {
        emulator.step();
        let sag = emulator.sag.as_ref().unwrap();
        results
            .get_mut("CalculatedTemperature")
            .unwrap()
            .push(sag.calculated_temperature);
        results
            .get_mut("TotalStrain")
            .unwrap()
            .push(sag.total_strain);
        results.get_mut("Sag").unwrap().push(sag.sag);
        step += 1;
    }

    // for _, field := range []string{"CalculatedTemperature", "TotalStrain", "Sag"} {
    // 	assert.IsType(t, []float64{}, results[field])
    // }
}
