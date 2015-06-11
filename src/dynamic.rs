//! 
//! A dynamic synth type.
//!

use mode;
use note_freq;
use synth;

pub use self::oscillator::Oscillator;
pub use self::oscillator::new as new_oscillator;


pub mod oscillator {
    use oscillator::Oscillator as Osc;
    pub use oscillator::waveform::Dynamic as Waveform;
    pub use oscillator::amplitude::Dynamic as Amplitude;
    pub use oscillator::frequency::Dynamic as Frequency;
    pub use oscillator::freq_warp::Dynamic as FreqWarp;

    /// An alias for a totally dynamic Oscillator.
    pub type Oscillator = Osc<Waveform, Amplitude, Frequency, FreqWarp>;

    /// Construct a new dynamic oscillator.
    pub fn new() -> Oscillator {
        use pitch::{LetterOctave, Letter};
        Oscillator::new(Waveform::Sine,
                        Amplitude::Constant(0.7),
                        Frequency::Hz(LetterOctave(Letter::C, 2).hz() as f64),
                        FreqWarp::None)
    }
}


/// An alias for a completely dynamic synth.
pub type SynthType = synth::Synth<mode::Dynamic,
                                  note_freq::DynamicGenerator,
                                  oscillator::Waveform,
                                  oscillator::Amplitude,
                                  oscillator::Frequency,
                                  oscillator::FreqWarp>;

/// An alias for a completely dynamic synth mode.
pub type Mode = mode::Dynamic;

/// An alias for a completely dynamic note freq generator.
pub type NoteFreqGenerator = note_freq::DynamicGenerator;

/// A wrapper for extending the functionality of a completely dynamic Synth.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Synth(pub SynthType);


impl ::std::ops::Deref for Synth {
    type Target = SynthType;
    fn deref<'a>(&'a self) -> &'a SynthType {
        &self.0
    }
}

impl ::std::ops::DerefMut for Synth {
    fn deref_mut<'a>(&'a mut self) -> &'a mut SynthType {
        &mut self.0
    }
}

impl<S> ::dsp::Node<S> for Synth where S: ::dsp::Sample {
    fn audio_requested(&mut self, output: &mut [S], settings: ::dsp::Settings) {
        ::dsp::Node::audio_requested(&mut self.0, output, settings)
    }
}


/// Construct the default dynamic synth.
pub fn new() -> SynthType {
    synth::Synth::new(mode::Dynamic::retrigger(), note_freq::DynamicGenerator::Constant)
}


impl Synth {

    /// Set the mode of the synth.
    pub fn set_mode(&mut self, mode: mode::Dynamic) {
        let Synth(ref mut synth) = *self;
        synth.mode = mode;
    }

    /// Set the note frequency generator to be used by the synth.
    pub fn set_note_freq_gen(&mut self, note_freq_gen: note_freq::DynamicGenerator) {
        let Synth(ref mut synth) = *self;
        synth.note_freq_gen = note_freq_gen;
    }

}


