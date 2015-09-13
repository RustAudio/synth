
use envelope;
use envelope::Trait as EnvelopeTrait;


/// Types for generating the amplitude given some playhead position.
pub trait Amplitude {
    /// Return the amplitude given some percentage through the duration of the Synth.
    /// - 0.0 < perc < 1.0.
    fn amp_at_playhead(&self, perc: f64) -> f32;
}

/// Alias for the Envelope used.
pub type Envelope = envelope::Envelope;

/// A type that allows dynamically switching between constant and enveloped amplitude.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Dynamic {
    Envelope(Envelope),
    Constant(f32),
}


impl Dynamic {
    /// Return whether or not the Dynamic is an Envelope.
    pub fn is_env(&self) -> bool {
        if let Dynamic::Envelope(_) = *self { true } else { false }
    }
}


impl Amplitude for f32 {
    #[inline]
    fn amp_at_playhead(&self, _perc: f64) -> f32 { *self }
}

impl Amplitude for Envelope {
    #[inline]
    fn amp_at_playhead(&self, perc: f64) -> f32 {
        self.y(perc).expect("The given playhead position is out of range (0.0..1.0).") as f32
    }
}

impl Amplitude for Dynamic {
    #[inline]
    fn amp_at_playhead(&self, perc: f64) -> f32 {
        match *self {
            Dynamic::Envelope(ref env) => env.amp_at_playhead(perc),
            Dynamic::Constant(amp) => amp,
        }
    }
}

