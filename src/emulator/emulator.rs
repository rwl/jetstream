use std::f64::consts::PI;

// Emulated event types
pub enum EventType {
    SinglePhaseFault,
    ThreePhaseFault,
    OverVoltage,
    UnderVoltage,
    OverFrequency,
    UnderFrequency,
    CapacitorOverCurrent,
}

/// The number of samples before initiating an emulated fault.
pub const EMULATED_FAULT_START_SAMPLES: usize = 1000;

/// The number of samples for emulating a fault.
pub const MAX_EMULATED_FAULT_DURATION_SAMPLES: usize = 6000;

/// The number of samples for emulating a fault.
pub const MAX_EMULATED_CAPACITOR_OVER_CURRENT_SAMPLES: usize = 8000;

/// The number of samples for emulating frequency deviations.
pub const MAX_EMULATED_FREQUENCY_DURATION_SAMPLES: usize = 8000;

/// The additional fault current magnitude added to one circuit end.
pub const EMULATED_FAULT_CURRENT_MAGNITUDE: usize = 80;

pub const TWO_PI_OVER_THREE: f64 = 2.0 * PI / 3.0;

pub struct ThreePhaseEmulation {
    // inputs
    pub pos_seq_mag: f64,
    pub phase_offset: f64,
    pub neg_seq_mag: f64,
    pub neg_seq_ang: f64,
    pub zero_seq_mag: f64,
    pub zero_seq_ang: f64,
    pub harmonic_numbers: Vec<f64>, //`mapstructure:",omitempty,flow"`
    pub harmonic_mags: Vec<f64>, //`mapstructure:",omitempty,flow"` // pu, relative to pos_seq_mag
    pub harmonic_angs: Vec<f64>, //`mapstructure:",omitempty,flow"`
    pub noise_max: f64,

    // event emulation
    pub fault_phase_a_mag: f64,
    pub fault_pos_seq_mag: f64,
    pub fault_remaining_samples: usize,

    // state change
    pub pos_seq_mag_new: f64,
    pub pos_seq_mag_ramp_rate: f64,

    // internal state
    p_angle: f64,

    // outputs
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Default for ThreePhaseEmulation {
    fn default() -> Self {
        Self {
            pos_seq_mag: 0.0,
            phase_offset: 0.0,
            neg_seq_mag: 0.0,
            neg_seq_ang: 0.0,
            zero_seq_mag: 0.0,
            zero_seq_ang: 0.0,
            harmonic_numbers: vec![],
            harmonic_mags: vec![],
            harmonic_angs: vec![],
            noise_max: 0.0,
            fault_phase_a_mag: 0.0,
            fault_pos_seq_mag: 0.0,
            fault_remaining_samples: 0,
            pos_seq_mag_new: 0.0,
            pos_seq_mag_ramp_rate: 0.0,
            p_angle: 0.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
        }
    }
}

pub struct TemperatureEmulation {
    pub mean_temperature: f64,
    pub noise_max: f64,
    pub modulation_mag: f64,

    // instantaneous anomalies
    pub(crate) is_instantaneous_anomaly: bool, // private
    pub instantaneous_anomaly_probability: f64,
    pub instantaneous_anomaly_magnitude: f64,

    // trend anomalies
    pub is_trend_anomaly: bool,
    pub trend_anomaly_duration: usize, // duration in seconds
    pub trend_anomaly_index: usize,
    pub trend_anomaly_magnitude: f64,

    pub is_rising_trend_anomaly: bool,

    pub t: f64,
}

pub struct SagEmulation {
    pub mean_strain: f64,
    pub mean_sag: f64,
    pub mean_calculated_temperature: f64,

    // outputs
    pub total_strain: f64,
    pub sag: f64,
    pub calculated_temperature: f64,
}

/// Encapsulates the waveform emulation of three-phase voltage, three-phase current, or temperature.
pub struct Emulator {
    // common inputs
    pub sampling_rate: usize,
    pub ts: f64,
    pub nom: f64,
    pub deviation: f64,

    pub v: Option<ThreePhaseEmulation>,
    pub i: Option<ThreePhaseEmulation>,

    pub t: Option<TemperatureEmulation>,
    pub sag: Option<SagEmulation>,

    // common state
    pub smp_cnt: usize,
    deviation_remaining_samples: usize,

    r: Box<dyn rand::Rng>,
}

// impl Default for Emulator {}

fn wrap_angle(a: f64) -> f64 {
    if a > PI {
        a - 2 * PI
    } else {
        a
    }
}

impl Emulator {
    /// Initiates an emulated event.
    pub fn start_event(&mut self, event_type: EventType) {
        // println!("StartEvent(): {}", event_type);

        match event_type {
            EventType::SinglePhaseFault => {
                let i = self.i.as_mut().unwrap();
                let v = self.v.as_mut().unwrap();
                // TODO
                // i.fault_pos_seq_mag = EMULATED_FAULT_CURRENT_MAGNITUDE
                // i.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES
                i.fault_phase_a_mag = i.pos_seq_mag * 1.2; //EMULATED_FAULT_CURRENT_MAGNITUDE
                i.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES;
                v.fault_phase_a_mag = v.pos_seq_mag * -0.2;
                v.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES;
            }
            EventType::ThreePhaseFault => {
                let i = self.i.as_mut().unwrap();
                let v = self.v.as_mut().unwrap();

                i.fault_pos_seq_mag = i.pos_seq_mag * 1.2; //EMULATED_FAULT_CURRENT_MAGNITUDE
                i.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES;
                v.fault_pos_seq_mag = v.pos_seq_mag * -0.2;
                v.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES;
            }
            EventType::OverVoltage => {
                let v = self.v.as_mut().unwrap();

                v.fault_pos_seq_mag = v.pos_seq_mag * 0.2;
                v.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES;
            }
            EventType::UnderVoltage => {
                let v = self.v.as_mut().unwrap();

                v.fault_pos_seq_mag = v.pos_seq_mag * -0.2;
                v.fault_remaining_samples = MAX_EMULATED_FAULT_DURATION_SAMPLES;
            }
            EventType::OverFrequency => {
                self.deviation = 0.1;
                self.deviation_remaining_samples = MAX_EMULATED_FREQUENCY_DURATION_SAMPLES;
            }
            EventType::UnderFrequency => {
                self.deviation = -0.1;
                self.deviation_remaining_samples = MAX_EMULATED_FREQUENCY_DURATION_SAMPLES;
            }
            EventType::CapacitorOverCurrent => {
                let i = self.i.as_mut().unwrap();
                todo!("CapacitorOverCurrent");
                i.fault_pos_seq_mag = i.pos_seq_mag * 0.01;
                i.fault_remaining_samples = MAX_EMULATED_CAPACITOR_OVER_CURRENT_SAMPLES;
            }
        }
    }

    pub fn new(sampling_rate: usize, frequency: f64) -> Self {
        Emulator {
            sampling_rate,
            nom: frequency,
            deviation: 0.0,
            ts: 1.0 / (sampling_rate as f64),

            v: None,
            i: None,
            t: None,
            sag: None,
            smp_cnt: 0,
            deviation_remaining_samples: 0,
            // r: rand.New(rand.NewSource(time.Now().Unix())),
            r: Box::new(rand::thread_rng()),
        }
    }

    /// Performs one iteration of the waveform generation.
    pub fn step(&mut self) {
        let f = self.nom + self.deviation;

        if self.deviation_remaining_samples > 0 {
            self.deviation_remaining_samples -= 1;
            if self.deviation_remaining_samples == 0 {
                self.deviation = 0.0;
            }
        }

        if let Some(v) = self.v.as_mut() {
            v.step_three_phase(&self.r, f, self.ts, self.smp_cnt);
        }
        if let Some(i) = self.i.as_mut() {
            i.step_three_phase(&self.r, f, self.ts, self.smp_cnt);
        }
        if let Some(t) = self.t.as_mut() {
            t.step_temperature(&self.r, self.ts);
        }
        if let Some(sag) = self.sag.as_mut() {
            sag.step_sag(&self.r);
        }

        self.smp_cnt += 1;
        if (self.smp_cnt as usize) >= self.sampling_rate {
            self.smp_cnt = 0
        }
    }
}

impl TemperatureEmulation {
    fn step_temperature(&mut self, r: &Box<dyn rand::Rng>, ts: f64) {
        let varying_t = self.mean_temperature * (1 + self.modulation_mag * f64::cos(1000.0 * ts));

        let mut trend_anomaly_delta = 0.0;
        let trend_anomaly_step =
            (self.trend_anomaly_magnitude / (self.trend_anomaly_duration as f64)) * ts;

        if self.is_trend_anomaly == true {
            if self.is_rising_trend_anomaly == true {
                trend_anomaly_delta = (self.trend_anomaly_index as f64) * trend_anomaly_step;
            } else {
                trend_anomaly_delta =
                    (self.trend_anomaly_index as f64) * trend_anomaly_step * (-1.0)
            }

            if self.trend_anomaly_index == int(float64(self.trend_anomaly_duration) / ts) - 1 {
                self.trend_anomaly_index = 0;
            } else {
                self.trend_anomaly_index += 1;
            }
        }

        let instantaneous_anomaly_delta = if self.instantaneous_anomaly_probability > rand.Float64()
        {
            self.is_instantaneous_anomaly = true;
            self.instantaneous_anomaly_magnitude
        } else {
            self.is_instantaneous_anomaly = false;
            0.0
        };

        let total_anomaly_delta = trend_anomaly_delta + instantaneous_anomaly_delta;

        self.t = varying_t
            + r.NormFloat64() * self.noise_max * self.mean_temperature
            + total_anomaly_delta;
    }
}

impl ThreePhaseEmulation {
    fn step_three_phase(&mut self, r: &Box<dyn rand::Rng>, f: f64, ts: f64, _smp_cnt: usize) {
        let angle = (f * 2.0 * PI * ts + self.p_angle);
        let angle = wrap_angle(angle);
        self.p_angle = angle;

        let pos_seq_phase = self.phase_offset + self.p_angle;

        if f64::abs(self.pos_seq_mag_new - self.pos_seq_mag) >= f64::abs(self.pos_seq_mag_ramp_rate)
        {
            self.pos_seq_mag = self.pos_seq_mag + self.pos_seq_mag_ramp_rate
        }

        let mut pos_seq_mag = self.pos_seq_mag;
        // phaseAMag := self.pos_seq_mag
        if
        /*smpCnt > EMULATED_FAULT_START_SAMPLES && */
        self.fault_remaining_samples > 0 {
            pos_seq_mag = pos_seq_mag + self.fault_pos_seq_mag;
            self.fault_remaining_samples -= 1;
        }

        // positive sequence
        let a1 = f64::sin(pos_seq_phase) * pos_seq_mag;
        let b1 = f64::sin(pos_seq_phase - TWO_PI_OVER_THREE) * pos_seq_mag;
        let c1 = f64::sin(pos_seq_phase + TWO_PI_OVER_THREE) * pos_seq_mag;

        // negative sequence
        let a2 = f64::sin(pos_seq_phase + self.neg_seq_ang) * self.neg_seq_mag * self.pos_seq_mag;
        let b2 = f64::sin(pos_seq_phase + TWO_PI_OVER_THREE + self.neg_seq_ang)
            * self.neg_seq_mag
            * self.pos_seq_mag;
        let c2 = f64::sin(pos_seq_phase - TWO_PI_OVER_THREE + self.neg_seq_ang)
            * self.neg_seq_mag
            * self.pos_seq_mag;

        // zero sequence
        let abc0 = f64::sin(pos_seq_phase + self.zero_seq_ang) * self.zero_seq_mag;

        // harmonics
        let mut ah = 0.0;
        let mut bh = 0.0;
        let mut ch = 0.0;
        if self.harmonic_numbers.len() > 0 {
            // ensure consistent array sizes have been specified
            if self.harmonic_numbers.len() == self.harmonic_mags.len()
                && self.harmonic_numbers.len() == self.harmonic_angs.len()
            {
                self.harmonic_numbers.iter().enumerate().for_each(|(i, n)| {
                    let mag = self.harmonic_mags[i] * self.pos_seq_mag;
                    let ang = self.harmonic_angs[i]; // / 180.0 * PI

                    ah = ah + f64::sin(n * (pos_seq_phase) + ang) * mag;
                    bh = bh + f64::sin(n * (pos_seq_phase - TWO_PI_OVER_THREE) + ang) * mag;
                    ch = ch + f64::sin(n * (pos_seq_phase + TWO_PI_OVER_THREE) + ang) * mag;
                });
            }
        }

        // add noise, ensure worst case where noise is uncorrelated across phases
        let ra = r.NormFloat64() * self.noise_max * self.pos_seq_mag;
        let rb = r.NormFloat64() * self.noise_max * self.pos_seq_mag;
        let rc = r.NormFloat64() * self.noise_max * self.pos_seq_mag;

        // combine the output for each phase
        self.a = a1 + a2 + abc0 + ah + ra;
        self.b = b1 + b2 + abc0 + bh + rb;
        self.c = c1 + c2 + abc0 + ch + rc;
    }
}

impl SagEmulation {
    fn step_sag(&mut self, r: &Box<dyn rand::Rng>) {
        r.Seed(time.Now().UnixNano());
        self.total_strain = self.mean_strain * r.Float64();
        self.sag = self.mean_sag * r.Float64();
        self.calculated_temperature = self.mean_calculated_temperature * r.Float64();
    }
}
