
/// An Oscillator must use one of a variety
/// of waveform types.
#[derive(Copy, Clone, Debug, Eq, PartialEq, RustcEncodable, RustcDecodable)]
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
    NoiseWalk
}

impl Waveform {

    /// Return the amplitude of a waveform at a given phase.
    pub fn amp_at_phase(&self, phase: f64) -> f32 {
        use std::f64::consts::PI_2;
        use std::num::Float;
        use utils::{fmod, noise_walk};
        let amp = match *self {
            Waveform::Sine => (PI_2 * phase).sin(),
            Waveform::Saw => fmod(phase, 1.0) * -2.0 + 1.0,
            Waveform::Square => if (PI_2 * phase).sin() < 0.0 { -1.0 } else { 1.0 },
            Waveform::Noise => ::rand::random::<f64>() * 2.0 - 1.0,
            Waveform::NoiseWalk => noise_walk(phase),
        };
        amp as f32
    }

}

