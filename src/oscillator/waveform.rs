//! 
//! The Waveform trait along with various Waveform Types and there implementations.
//!

/// Some type that can return an amplitude given some phase.
pub trait Waveform {
    /// Return the amplitude given some phase.
    fn amp_at_phase(&self, phase: f64) -> f32;
}


/// Twice PI.
const PI_2: f64 = ::std::f64::consts::PI * 2.0;

/// Represents the "steepness" of the exponential saw wave.
pub type Steepness = f32;

/// An Oscillator must use one of a variety
/// of waveform types.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Dynamic {
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

/// A sine wave.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Sine;

/// A sawtooth wave.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Saw;

/// An exponential sawtooth wave.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct SawExp(pub Steepness);

/// A square wave.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Square;

/// A noise signal.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Noise;

/// A random noise walk wave.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct NoiseWalk;


impl Waveform for Dynamic {
    /// Return the amplitude of a waveform at a given phase.
    #[inline]
    fn amp_at_phase(&self, phase: f64) -> f32 {
        match *self {
            Dynamic::Sine => Sine.amp_at_phase(phase),
            Dynamic::Saw => Saw.amp_at_phase(phase),
            Dynamic::Square => Square.amp_at_phase(phase),
            Dynamic::Noise => Noise.amp_at_phase(phase),
            Dynamic::NoiseWalk => NoiseWalk.amp_at_phase(phase),
            Dynamic::SawExp(steepness) => SawExp(steepness).amp_at_phase(phase),
        }
    }
}

impl Waveform for Sine {
    #[inline]
    fn amp_at_phase(&self, phase: f64) -> f32 {
        (PI_2 * phase).sin() as f32
    }
}

impl Waveform for Saw {
    #[inline]
    fn amp_at_phase(&self, phase: f64) -> f32 {
        (::utils::fmod(phase, 1.0) * -2.0 + 1.0) as f32
    }
}

impl Waveform for SawExp {
    #[inline]
    fn amp_at_phase(&self, phase: f64) -> f32 {
        let SawExp(steepness) = *self;
        let saw = Saw.amp_at_phase(phase);
        saw * saw.abs().powf(steepness)
    }
}

impl Waveform for Square {
    #[inline]
    fn amp_at_phase(&self, phase: f64) -> f32 {
        (if ::utils::fmod(phase, 1.0) < 0.5 { -1.0 } else { 1.0 }) as f32
        //(if (PI_2 * phase).sin() < 0.0 { -1.0 } else { 1.0 }) as f32
    }
}

impl Waveform for Noise {
    #[inline]
    fn amp_at_phase(&self, _phase: f64) -> f32 {
        ::rand::random::<f32>() * 2.0 - 1.0
    }
}

impl Waveform for NoiseWalk {
    #[inline]
    fn amp_at_phase(&self, phase: f64) -> f32 {
        ::utils::noise_walk(phase as f32)
    }
}

