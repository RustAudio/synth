
//! Implementation of the `Synth` struct for basic polyphonic, multi-oscillator envelope synthesis.

extern crate dsp;
extern crate envelope;
extern crate gaussian;
extern crate num;
extern crate panning;
extern crate pitch_calc as pitch;
extern crate time_calc as time;
extern crate rand;
extern crate rustc_serialize;
extern crate utils;

pub use dynamic::Synth as Dynamic;
pub use env_point::Point;
pub use note_freq::{NoteFreqGenerator, NoteFreq, Portamento, PortamentoFreq};
pub use oscillator::{AmpEnvelope, FreqEnvelope, Oscillator, Waveform};
pub use synth::Synth;
pub use voice::Voice;

pub mod dynamic;
mod env_point;
pub mod mode;
pub mod note_freq;
pub mod oscillator;
mod synth;
mod voice;

