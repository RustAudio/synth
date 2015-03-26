
//! Implementation of the `Synth` struct for basic polyphonic, multi-oscillator envelope synthesis.

#![feature(core)]

extern crate dsp;
extern crate envelope;
extern crate gaussian;
extern crate pitch_calc as pitch;
extern crate time_calc as time;
extern crate rand;
extern crate rustc_serialize;
extern crate utils;

pub use env_point::Point;
pub use oscillator::{AmpEnvelope, FreqEnvelope, Oscillator};
pub use synth::Synth;
pub use voice::Voice;
pub use waveform::Waveform;

mod env_point;
mod oscillator;
pub mod synth;
pub mod voice;
pub mod waveform;

