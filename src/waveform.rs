
/// An Oscillator must use one of a variety
/// of waveform types.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Waveform {
    /// Sine Wave
    Sine,
    /// Saw Wave
    Saw,
    /// Square Wave
    Square,
    /// Noise
    Noise,
    /// Noise Walk
    NoiseWalk,
    /// Exponential Saw Wave.
    SawExp(Steepness),
}

/// Represents the "steepness" of the exponential saw wave.
pub type Steepness = f32;

impl Waveform {

    /// Return the amplitude of a waveform at a given phase.
    pub fn amp_at_phase(&self, phase: f64) -> f32 {
        use num::Float;
        use std::f64::consts::PI;
        use utils::{fmod, noise_walk};
        const PI_2: f64 = PI * 2.0;
        let amp = match *self {
            Waveform::Sine => (PI_2 * phase).sin(),
            Waveform::Saw => fmod(phase, 1.0) * -2.0 + 1.0,
            Waveform::Square => if (PI_2 * phase).sin() < 0.0 { -1.0 } else { 1.0 },
            Waveform::Noise => ::rand::random::<f64>() * 2.0 - 1.0,
            Waveform::NoiseWalk => noise_walk(phase),
            Waveform::SawExp(steepness) => {
                let saw = fmod(phase, 1.0) * -2.0 + 1.0;
                saw * saw.abs().powf(steepness as f64)
            },
        };
        amp as f32
    }

}

