
use pitch;
use super::waveform::{self, Waveform};


/// Types that produce a warped frequency in hz for some given frequency in hz.
pub trait FreqWarp {
    /// Step the phase of the frequency warp if necessary.
    fn step_phase(&self, _sample_hz: f64, _freq_warp_phase: &mut f64) {}
    /// Return a warped hz given some hz, sample rate and phase.
    fn warp_hz(&self, hz: f64, freq_warp_phase: f64) -> f64;
}

/// A type for warping the frequency via gaussian randomness.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gaussian(pub f32);

/// A type for slowly drifting an oscillators pitch via a noise walk.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PitchDrift {
    /// The frequncy at which the pitch should drift.
    pub hz: f64,
    /// How much the pitch should drift in steps.
    pub amp: f32,
}

/// A type that allows switching between various kinds of FreqWarp at runtime.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dynamic {
    None,
    Gaussian(Gaussian),
    PitchDrift(PitchDrift),
}


impl Dynamic {
    /// Construct a gaussian.
    pub fn gaussian(amt: f32) -> Dynamic {
        Dynamic::Gaussian(Gaussian(amt))
    }
    /// Construct a pitch drift.
    pub fn pitch_drift(amp: f32, hz: f64) -> Dynamic {
        Dynamic::PitchDrift(PitchDrift { amp: amp, hz: hz })
    }
}


impl FreqWarp for () {
    #[inline]
    fn warp_hz(&self, hz: f64, _freq_warp_phase: f64) -> f64 { hz }
}

impl FreqWarp for Gaussian {
    #[inline]
    fn warp_hz(&self, hz: f64, _freq_warp_phase: f64) -> f64 {
        let Gaussian(perc) = *self;
        if perc > 0.0 {
            use gaussian;
            let mels = pitch::Hz(hz as f32).mel();
            let gaus_mels = mels + gaussian::gen(0.5f32, perc.powf(2.0)) * 1000.0 - 500.0;
            pitch::Mel(gaus_mels).hz() as f64
        } else {
            hz
        }
    }
}

impl FreqWarp for PitchDrift {
    #[inline]
    fn step_phase(&self, sample_hz: f64, freq_warp_phase: &mut f64) {
        *freq_warp_phase = *freq_warp_phase + self.hz / sample_hz;
    }
    #[inline]
    fn warp_hz(&self, hz: f64, freq_warp_phase: f64) -> f64 {
        let offset_in_steps = waveform::NoiseWalk.amp_at_phase(freq_warp_phase) * self.amp;
        let warped_hz = pitch::Step(pitch::Hz(hz as f32).step() + offset_in_steps).hz() as f64;
        warped_hz
    }
}

impl FreqWarp for Dynamic {
    #[inline]
    fn step_phase(&self, sample_hz: f64, freq_warp_phase: &mut f64) {
        match *self {
            Dynamic::None                        |
            Dynamic::Gaussian(_)                 => (),
            Dynamic::PitchDrift(ref pitch_drift) => pitch_drift.step_phase(sample_hz, freq_warp_phase),
        }
    }
    #[inline]
    fn warp_hz(&self, hz: f64, freq_warp_phase: f64) -> f64 {
        match *self {
            Dynamic::None                        => hz,
            Dynamic::Gaussian(ref gaussian)      => gaussian.warp_hz(hz, freq_warp_phase),
            Dynamic::PitchDrift(ref pitch_drift) => pitch_drift.warp_hz(hz, freq_warp_phase),
        }
    }
}

