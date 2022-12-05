use crate::emulator::{Emulator, SagEmulation, TemperatureEmulation, ThreePhaseEmulation};
use std::collections::HashMap;
use std::f64::consts::PI;

// benchmark emulator performance
fn BenchmarkEmulator(b: testing::B) {
    let mut emu = createEmulatorForBenchmark(4000, 0.0);

    for i in 0..b.N {
        for j in 0..4000 {
            emu.Step();
        }
    }
}

fn createEmulatorForBenchmark(samplingRate: usize, phaseOffsetDeg: f64) -> Emulator {
    let mut emu = Emulator::new(samplingRate, 50.0);

    emu.V = Some(ThreePhaseEmulation {
        PosSeqMag: 400000.0 / math.Sqrt(3) * math.Sqrt(2),
        NoiseMax: 0.000001,
        PhaseOffset: phaseOffsetDeg * PI / 180.0,
        ..Default::default()
    });
    emu.I = Some(ThreePhaseEmulation {
        PosSeqMag: 500.0,
        PhaseOffset: phaseOffsetDeg * PI / 180.0,
        HarmonicNumbers: vec![5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0, 25.0],
        HarmonicMags: vec![
            0.2164, 0.1242, 0.0892, 0.0693, 0.0541, 0.0458, 0.0370, 0.0332,
        ],
        HarmonicAngs: vec![171.5, 100.4, -52.4, 128.3, 80.0, 2.9, -146.8, 133.9],
        NoiseMax: 0.000001,
        ..Default::default()
    });

    emu
}

fn createEmulator(samplingRate: usize, phaseOffsetDeg: f64) -> Emulator {
    let mut emu = Emulator::new(samplingRate, 50.0);

    emu.V = Some(ThreePhaseEmulation {
        PosSeqMag: 400000.0 / math.Sqrt(3) * math.Sqrt(2),
        NoiseMax: 0.000001,
        PhaseOffset: phaseOffsetDeg * PI / 180.0,
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
        NoiseMax: 0.000001,
        ..Default::default()
    });
    emu.T = Some(TemperatureEmulation {
        MeanTemperature: 30.0,
        NoiseMax: 0.01,
        InstantaneousAnomalyMagnitude: 30.0,
        InstantaneousAnomalyProbability: 0.01,
        ..Default::default()
    });
    emu
}

fn FloatingPointEqual(expected: f64, actual: f64, threshold: f64) -> bool {
    let absDiff = (expected - actual).abs();
    absDiff < threshold
}

fn mean(values: &[f64]) -> f64 {
    let mut sum: f64 = 0.0;
    values.iter().for_each(|value| sum += value);
    sum / (values.len() as f64)
}

fn TestTemperatureEmulationAnomalies_NoAnomalies(t: testing::T) {
    let mut emulator = createEmulator(14400, 0.0);

    emulator.T.as_mut().unwrap().InstantaneousAnomalyProbability = 0.0;
    let mut step = 0;
    let mut results: Vec<bool> = vec![];
    while step < 1e4 {
        emulator.Step();
        results.push(emulator.T.isInstantaneousAnomaly);
        step += 1;
    }
    assert.NotContains(t, results, true);
}

fn TestTemperatureEmulationAnomalies_Anomalies(t: testing::T) {
    let mut emulator = createEmulator(14400, 0.0);

    emulator.T.as_mut().unwrap().InstantaneousAnomalyProbability = 0.5;
    let mut step = 0;
    let mut results: Vec<bool> = vec![];
    let mut normalValues: Vec<f64> = vec![];
    let mut anomalyValues: Vec<f64> = vec![];
    while step < 1e4 {
        emulator.Step();
        results.push(emulator.T.unwrap().isInstantaneousAnomaly);

        if emulator.T.unwrap().isInstantaneousAnomaly == true {
            anomalyValues.push(emulator.T.unwrap().T);
        } else {
            normalValues.push(emulator.T.unwrap().T);
        }
        step += 1;
    }
    assert.Contains(t, results, true);

    let fractionAnomalies = (anomalyValues.len() as f64) / (step as f64);
    assert.True(t, FloatingPointEqual(0.5, fractionAnomalies, 0.1));

    assert.True(t, mean(&anomalyValues) > mean(&normalValues));
}

fn TestTemperatureEmulationAnomalies_RisingTrend(t: testing::T) {
    let mut emulator = createEmulator(14400, 0.0);
    emulator.T.as_mut().unwrap().IsTrendAnomaly = true;
    emulator.T.as_mut().unwrap().TrendAnomalyMagnitude = 30.0;
    emulator.T.as_mut().unwrap().TrendAnomalyDuration = 10;
    emulator.T.as_mut().unwrap().IsRisingTrendAnomaly = true;

    let mut step = 0;
    let mut results: Vec<f64> = vec![];
    while step < emulator.T.unwrap().TrendAnomalyDuration * emulator.SamplingRate {
        emulator.Step();
        results.push(emulator.T.unwrap().T);
        step += 1;

        if step < emulator.SamplingRate {
            assert.NotEqual(t, 0, emulator.T.unwrap().TrendAnomalyIndex);
        }
    }

    assert.True(t, mean(&results) > emulator.T.MeanTemperature);
}

fn TestTemperatureEmulationAnomalies_DecreasingTrend(t: testing::T) {
    let mut emulator = createEmulator(1, 0.0);
    emulator.T.as_mut().unwrap().IsTrendAnomaly = true;
    emulator.T.as_mut().unwrap().TrendAnomalyMagnitude = 30.0;
    emulator.T.as_mut().unwrap().TrendAnomalyDuration = 10;
    emulator.T.as_mut().unwrap().IsRisingTrendAnomaly = false;
    let mut step = 0;
    let mut results: Vec<f64> = vec![];
    while step < 10 {
        emulator.Step();
        results.push(emulator.T.unwrap().T);
        step += 1;
    }

    assert.True(t, mean(&results) < emulator.T.unwrap().MeanTemperature);
}

fn TestSagEmulation(t: testing::T) {
    let mut emulator = createEmulator(14400, 0.0);
    emulator.Sag = Some(SagEmulation {
        MeanCalculatedTemperature: 30.0,
        MeanStrain: 100.0,
        MeanSag: 0.5,
        ..Default::default()
    });

    let mut step = 0;
    let mut results = HashMap::<String, Vec<f64>>::new();
    results["CalculatedTemperature"] = vec![];
    results["TotalStrain"] = vec![];
    results["Sag"] = vec![];

    while step < 1e4 {
        emulator.Step();
        results["CalculatedTemperature"].push(emulator.Sag.CalculatedTemperature);
        results["TotalStrain"].push(emulator.Sag.CalculatedTemperature);
        results["Sag"].push(emulator.Sag.CalculatedTemperature);
        step += 1;
    }

    // for _, field := range []string{"CalculatedTemperature", "TotalStrain", "Sag"} {
    // 	assert.IsType(t, []float64{}, results[field])
    // }
}
