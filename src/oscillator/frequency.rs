use envelope;
use pitch;


/// Types for generating the frequency given some playhead position.
pub trait Frequency {
    /// Return the frequency given some playhead percentage through the duration of the Synth.
    /// - 0.0 < perc < 1.0l
    fn hz_at_playhead(&self, perc: f64) -> f64;
    /// Return the frequency as a percentage.
    #[inline]
    fn freq_perc_at_playhead(&self, perc: f64) -> f64 {
        pitch::Hz(self.hz_at_playhead(perc) as f32).perc()
    }
}

/// Alias for the Envelope used.
pub type Envelope = envelope::Envelope;

/// A type that allows dynamically switching between constant and enveloped frequency.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Envelope(Envelope),
    Hz(f64),
}


impl Dynamic {

    /// Return whether or not the Dynamic is an Envelope.
    pub fn is_env(&self) -> bool {
        if let Dynamic::Envelope(_) = *self { true } else { false }
    }

    /// Convert the dynamic to its Envelope variant.
    pub fn to_env(&self) -> Dynamic {
        use std::iter::once;
        if let Dynamic::Hz(hz) = *self {
            let perc = pitch::Hz(hz as f32).perc();
            return Dynamic::Envelope({
                once(envelope::Point::new(0.0, perc, 0.0))
                    .chain(once(envelope::Point::new(1.0, perc, 0.0)))
                    .collect()
            })
        }
        self.clone()
    }

    /// Convert the dynamic to its Hz variant.
    pub fn to_hz(&self) -> Dynamic {
        if let Dynamic::Envelope(ref env) = *self {
            use pitch::{LetterOctave, Letter};
            // Just convert the first point to the constant Hz.
            return match env.points.iter().nth(0) {
                Some(point) => Dynamic::Hz(pitch::Perc(point.y).hz() as f64),
                None => Dynamic::Hz(LetterOctave(Letter::C, 1).hz() as f64),
            }
        }
        self.clone()
    }
}


impl Frequency for f64 {
    #[inline]
    fn hz_at_playhead(&self, _perc: f64) -> f64 { *self }
}

impl Frequency for Envelope {
    #[inline]
    fn hz_at_playhead(&self, perc: f64) -> f64 {
        pitch::Perc(self.freq_perc_at_playhead(perc)).hz() as f64
    }
    #[inline]
    fn freq_perc_at_playhead(&self, perc: f64) -> f64 {
        envelope::Trait::y(self, perc)
            .expect("The given playhead position is out of range (0.0..1.0).")
    }
}

impl Frequency for Dynamic {
    #[inline]
    fn hz_at_playhead(&self, perc: f64) -> f64 {
        match *self {
            Dynamic::Envelope(ref env) => env.hz_at_playhead(perc),
            Dynamic::Hz(hz) => hz,
        }
    }
}
