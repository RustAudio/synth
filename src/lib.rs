//! Implementation of the `Synth` struct for basic polyphonic, multi-oscillator envelope synthesis.

extern crate dsp;
extern crate envelope as envelope_lib;
extern crate gaussian;
extern crate instrument;
extern crate num;
extern crate panning;
extern crate pitch_calc as pitch;
extern crate time_calc as time;
extern crate rand;
extern crate rustc_serialize;
extern crate utils;

pub use dynamic::Synth as Dynamic;
pub use envelope::{Envelope, Point};
pub use envelope::Trait as EnvelopeTrait;
pub use note_freq::{NoteFreqGenerator, NoteFreq, Portamento, PortamentoFreq};
pub use oscillator::{AmpEnvelope, FreqEnvelope, Oscillator, Waveform};
pub use synth::Synth;
pub use voice::Voice;

pub mod dynamic;
pub mod envelope;
pub mod mode;
pub mod note_freq;
pub mod oscillator;
mod synth;
mod voice;
