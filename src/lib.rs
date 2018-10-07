//! Implementation of the `Synth` struct for basic polyphonic, multi-oscillator envelope synthesis.

extern crate envelope as envelope_lib;
extern crate gaussian;
pub extern crate instrument;
extern crate panning;
extern crate pitch_calc as pitch;
extern crate time_calc as time;
extern crate rand;
extern crate sample;
extern crate utils;

pub use dynamic::Synth as Dynamic;
pub use envelope::{Envelope, Point};
pub use envelope::Trait as EnvelopeTrait;
pub use oscillator::{AmpEnvelope, FreqEnvelope, Oscillator, Waveform};
pub use synth::{Synth, Frames};

pub mod dynamic;
pub mod envelope;
pub mod oscillator;
mod synth;

#[cfg(feature="dsp-chain")]
mod dsp_node;

#[cfg(feature="serde_serialization")]
mod serde;
