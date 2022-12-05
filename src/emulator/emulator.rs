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
pub const EmulatedFaultStartSamples: usize = 1000;

/// The number of samples for emulating a fault.
pub const MaxEmulatedFaultDurationSamples: usize = 6000;

/// The number of samples for emulating a fault.
pub const MaxEmulatedCapacitorOverCurrentSamples: usize = 8000;

/// The number of samples for emulating frequency deviations.
pub const MaxEmulatedFrequencyDurationSamples: usize = 8000;

/// The additional fault current magnitude added to one circuit end.
pub const EmulatedFaultCurrentMagnitude: usize = 80;

pub const TwoPiOverThree: f64 = 2.0 * PI / 3.0;

pub struct ThreePhaseEmulation {
    // inputs
    pub PosSeqMag: f64,
    pub PhaseOffset: f64,
    pub NegSeqMag: f64,
    pub NegSeqAng: f64,
    pub ZeroSeqMag: f64,
    pub ZeroSeqAng: f64,
    pub HarmonicNumbers: Vec<f64>, //`mapstructure:",omitempty,flow"`
    pub HarmonicMags: Vec<f64>,    //`mapstructure:",omitempty,flow"` // pu, relative to PosSeqMag
    pub HarmonicAngs: Vec<f64>,    //`mapstructure:",omitempty,flow"`
    pub NoiseMax: f64,

    // event emulation
    pub FaultPhaseAMag: f64,
    pub FaultPosSeqMag: f64,
    pub FaultRemainingSamples: usize,

    // state change
    pub PosSeqMagNew: f64,
    pub PosSeqMagRampRate: f64,

    // internal state
    pAngle: f64,

    // outputs
    pub A: f64,
    pub B: f64,
    pub C: f64,
}

impl Default for ThreePhaseEmulation {
    fn default() -> Self {
        Self {
            PosSeqMag: 0.0,
            PhaseOffset: 0.0,
            NegSeqMag: 0.0,
            NegSeqAng: 0.0,
            ZeroSeqMag: 0.0,
            ZeroSeqAng: 0.0,
            HarmonicNumbers: vec![],
            HarmonicMags: vec![],
            HarmonicAngs: vec![],
            NoiseMax: 0.0,
            FaultPhaseAMag: 0.0,
            FaultPosSeqMag: 0.0,
            FaultRemainingSamples: 0,
            PosSeqMagNew: 0.0,
            PosSeqMagRampRate: 0.0,
            pAngle: 0.0,
            A: 0.0,
            B: 0.0,
            C: 0.0,
        }
    }
}

pub struct TemperatureEmulation {
    pub MeanTemperature: f64,
    pub NoiseMax: f64,
    pub ModulationMag: f64,

    // instantaneous anomalies
    pub(crate) isInstantaneousAnomaly: bool, // private
    pub InstantaneousAnomalyProbability: f64,
    pub InstantaneousAnomalyMagnitude: f64,

    // trend anomalies
    pub IsTrendAnomaly: bool,
    pub TrendAnomalyDuration: usize, // duration in seconds
    pub TrendAnomalyIndex: usize,
    pub TrendAnomalyMagnitude: f64,

    pub IsRisingTrendAnomaly: bool,

    pub T: f64,
}

pub struct SagEmulation {
    pub MeanStrain: f64,
    pub MeanSag: f64,
    pub MeanCalculatedTemperature: f64,

    // outputs
    pub TotalStrain: f64,
    pub Sag: f64,
    pub CalculatedTemperature: f64,
}

/// Encapsulates the waveform emulation of three-phase voltage, three-phase current, or temperature.
pub struct Emulator {
    // common inputs
    pub SamplingRate: usize,
    pub Ts: f64,
    pub Fnom: f64,
    pub Fdeviation: f64,

    pub V: Option<ThreePhaseEmulation>,
    pub I: Option<ThreePhaseEmulation>,

    pub T: Option<TemperatureEmulation>,
    pub Sag: Option<SagEmulation>,

    // common state
    pub SmpCnt: usize,
    fDeviationRemainingSamples: usize,

    r: Box<dyn rand::Rng>,
}

// impl Default for Emulator {}

fn wrapAngle(a: f64) -> f64 {
    if a > PI {
        a - 2 * PI
    } else {
        a
    }
}

impl Emulator {
    /// Initiates an emulated event.
    pub fn StartEvent(&mut self, eventType: EventType) {
        // println!("StartEvent(): {}", eventType);

        match eventType {
            EventType::SinglePhaseFault => {
                // TODO
                // e.I.FaultPosSeqMag = EmulatedFaultCurrentMagnitude
                // e.I.FaultRemainingSamples = MaxEmulatedFaultDurationSamples
                e.I.FaultPhaseAMag = e.I.PosSeqMag * 1.2; //EmulatedFaultCurrentMagnitude
                e.I.FaultRemainingSamples = MaxEmulatedFaultDurationSamples;
                e.V.FaultPhaseAMag = e.V.PosSeqMag * -0.2;
                e.V.FaultRemainingSamples = MaxEmulatedFaultDurationSamples;
            }
            EventType::ThreePhaseFault => {
                e.I.FaultPosSeqMag = e.I.PosSeqMag * 1.2; //EmulatedFaultCurrentMagnitude
                e.I.FaultRemainingSamples = MaxEmulatedFaultDurationSamples;
                e.V.FaultPosSeqMag = e.V.PosSeqMag * -0.2;
                e.V.FaultRemainingSamples = MaxEmulatedFaultDurationSamples;
            }
            EventType::OverVoltage => {
                e.V.FaultPosSeqMag = e.V.PosSeqMag * 0.2;
                e.V.FaultRemainingSamples = MaxEmulatedFaultDurationSamples;
            }
            EventType::UnderVoltage => {
                e.V.FaultPosSeqMag = e.V.PosSeqMag * -0.2;
                e.V.FaultRemainingSamples = MaxEmulatedFaultDurationSamples;
            }
            EventType::OverFrequency => {
                e.Fdeviation = 0.1;
                e.fDeviationRemainingSamples = MaxEmulatedFrequencyDurationSamples;
            }
            EventType::UnderFrequency => {
                e.Fdeviation = -0.1;
                e.fDeviationRemainingSamples = MaxEmulatedFrequencyDurationSamples;
            }
            EventType::CapacitorOverCurrent => {
                todo!("CapacitorOverCurrent");
                e.I.FaultPosSeqMag = e.I.PosSeqMag * 0.01;
                e.I.FaultRemainingSamples = MaxEmulatedCapacitorOverCurrentSamples;
            }
        }
    }

    pub fn new(samplingRate: usize, frequency: f64) -> Self {
        Emulator {
            SamplingRate: samplingRate,
            Fnom: frequency,
            Fdeviation: 0.0,
            Ts: 1.0 / (samplingRate as f64),

            V: None,
            I: None,
            T: None,
            Sag: None,
            SmpCnt: 0,
            fDeviationRemainingSamples: 0,
            // r: rand.New(rand.NewSource(time.Now().Unix())),
            r: rand::thread_rng(),
        }
    }

    /// Performs one iteration of the waveform generation.
    pub fn Step(&mut self) {
        let f = self.Fnom + self.Fdeviation;

        if self.fDeviationRemainingSamples > 0 {
            self.fDeviationRemainingSamples -= 1;
            if self.fDeviationRemainingSamples == 0 {
                self.Fdeviation = 0.0;
            }
        }

        if let Some(v) = self.V.as_mut() {
            v.stepThreePhase(self.r, f, self.Ts, self.SmpCnt);
        }
        if let Some(i) = self.I.as_mut() {
            i.stepThreePhase(self.r, f, self.Ts, self.SmpCnt);
        }
        if let Some(t) = self.T.as_mut() {
            t.stepTemperature(self.r, self.Ts);
        }
        if let Some(sag) = self.Sag.as_mut() {
            sag.stepSag(self.r);
        }

        self.SmpCnt += 1;
        if (self.SmpCnt as usize) >= self.SamplingRate {
            self.SmpCnt = 0
        }
    }
}

impl TemperatureEmulation {
    fn stepTemperature(&mut self, r: rand::Rand, Ts: f64) {
        let varyingT = self.MeanTemperature * (1 + self.ModulationMag * f64::cos(1000.0 * Ts));

        let mut trendAnomalyDelta = 0.0;
        let trendAnomalyStep =
            (self.TrendAnomalyMagnitude / float64(self.TrendAnomalyDuration)) * Ts;

        if self.IsTrendAnomaly == true {
            if self.IsRisingTrendAnomaly == true {
                trendAnomalyDelta = (self.TrendAnomalyIndex as f64) * trendAnomalyStep;
            } else {
                trendAnomalyDelta = (self.TrendAnomalyIndex as f64) * trendAnomalyStep * (-1.0)
            }

            if self.TrendAnomalyIndex == int(float64(self.TrendAnomalyDuration) / Ts) - 1 {
                self.TrendAnomalyIndex = 0;
            } else {
                self.TrendAnomalyIndex += 1;
            }
        }

        let instantaneousAnomalyDelta = if self.InstantaneousAnomalyProbability > rand.Float64() {
            self.isInstantaneousAnomaly = true;
            self.InstantaneousAnomalyMagnitude
        } else {
            self.isInstantaneousAnomaly = false;
            0.0
        };

        let totalAnomalyDelta = trendAnomalyDelta + instantaneousAnomalyDelta;

        self.T =
            varyingT + r.NormFloat64() * self.NoiseMax * self.MeanTemperature + totalAnomalyDelta;
    }
}

impl ThreePhaseEmulation {
    fn stepThreePhase(&mut self, r: rand::Rand, f: f64, Ts: f64, smpCnt: usize) {
        let angle = (f * 2.0 * PI * Ts + self.pAngle);
        let angle = wrapAngle(angle);
        self.pAngle = angle;

        let PosSeqPhase = self.PhaseOffset + self.pAngle;

        if f64::abs(self.PosSeqMagNew - self.PosSeqMag) >= f64::abs(self.PosSeqMagRampRate) {
            self.PosSeqMag = self.PosSeqMag + self.PosSeqMagRampRate
        }

        let mut posSeqMag = self.PosSeqMag;
        // phaseAMag := self.PosSeqMag
        if
        /*smpCnt > EmulatedFaultStartSamples && */
        self.FaultRemainingSamples > 0 {
            posSeqMag = posSeqMag + self.FaultPosSeqMag;
            self.FaultRemainingSamples -= 1;
        }

        // positive sequence
        let a1 = f64::sin(PosSeqPhase) * posSeqMag;
        let b1 = f64::sin(PosSeqPhase - TwoPiOverThree) * posSeqMag;
        let c1 = f64::sin(PosSeqPhase + TwoPiOverThree) * posSeqMag;

        // negative sequence
        let a2 = f64::sin(PosSeqPhase + self.NegSeqAng) * self.NegSeqMag * self.PosSeqMag;
        let b2 = f64::sin(PosSeqPhase + TwoPiOverThree + self.NegSeqAng)
            * self.NegSeqMag
            * self.PosSeqMag;
        let c2 = f64::sin(PosSeqPhase - TwoPiOverThree + self.NegSeqAng)
            * self.NegSeqMag
            * self.PosSeqMag;

        // zero sequence
        let abc0 = f64::sin(PosSeqPhase + self.ZeroSeqAng) * self.ZeroSeqMag;

        // harmonics
        let mut ah = 0.0;
        let mut bh = 0.0;
        let mut ch = 0.0;
        if self.HarmonicNumbers.len() > 0 {
            // ensure consistent array sizes have been specified
            if self.HarmonicNumbers.len() == self.HarmonicMags.len()
                && self.HarmonicNumbers.len() == self.HarmonicAngs.len()
            {
                self.HarmonicNumbers.iter().enumerate().for_each(|(i, n)| {
                    let mag = self.HarmonicMags[i] * self.PosSeqMag;
                    let ang = self.HarmonicAngs[i]; // / 180.0 * PI

                    ah = ah + f64::sin(n * (PosSeqPhase) + ang) * mag;
                    bh = bh + f64::sin(n * (PosSeqPhase - TwoPiOverThree) + ang) * mag;
                    ch = ch + f64::sin(n * (PosSeqPhase + TwoPiOverThree) + ang) * mag;
                });
            }
        }

        // add noise, ensure worst case where noise is uncorrelated across phases
        let ra = r.NormFloat64() * self.NoiseMax * self.PosSeqMag;
        let rb = r.NormFloat64() * self.NoiseMax * self.PosSeqMag;
        let rc = r.NormFloat64() * self.NoiseMax * self.PosSeqMag;

        // combine the output for each phase
        self.A = a1 + a2 + abc0 + ah + ra;
        self.B = b1 + b2 + abc0 + bh + rb;
        self.C = c1 + c2 + abc0 + ch + rc;
    }
}

impl SagEmulation {
    fn stepSag(&mut self, r: rand::Rand) {
        r.Seed(time.Now().UnixNano());
        self.TotalStrain = self.MeanStrain * r.Float64();
        self.Sag = self.MeanSag * r.Float64();
        self.CalculatedTemperature = self.MeanCalculatedTemperature * r.Float64();
    }
}
