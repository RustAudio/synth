
/// An Oscillator must use one of a variety
/// of waveform types.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
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

    /// Return the new phrase considering the frequency, sample rate and waveform.
    pub fn next_phase(&self, current_phase: f64, freq: f64, sample_hz: f64) -> f64 {
        use utils::remainder;
        let advance_per_sample = freq / sample_hz;
        match *self {
            Waveform::Sine      |
            Waveform::NoiseWalk |
            Waveform::Square    => current_phase + advance_per_sample,
            Waveform::Saw   |
            Waveform::Noise => remainder(current_phase + advance_per_sample, 2.0),
        }
    }

    /// Return the amplitude of a waveform at a given phase.
    pub fn amp_at_phase(&self, phase: f64) -> f32 {
        use std::f64::consts::{PI, PI_2};
        use std::num::Float;
        use utils::{fmod, noise_walk};
        let amp = match *self {
            Waveform::Sine => (PI_2 * phase).sin(),
            Waveform::Saw => fmod(phase, 1.0) * -2.0 + 1.0,
            Waveform::Square => if phase < PI { -1.0 } else { 1.0 },
            Waveform::Noise => ::rand::random::<f64>() * 2.0 - 1.0,
            Waveform::NoiseWalk => noise_walk(phase),
        };
        amp as f32
    }

}

