
//! Synthesis Oscillator module.

use env_point::Point;
use pitch::{MAX_HZ, MIN_HZ};
use envelope::Envelope;
use waveform::Waveform;

pub type AmpEnvelope = Envelope<Point>;
pub type FreqEnvelope = Envelope<Point>;

/// The fundamental component of a synthesizer.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Oscillator {
    phase: f64,
    /// Waveform used for phase movement.
    pub waveform: Waveform,
    /// The percentage of randomness to be applied to freq.
    pub gaussian_perc: f32,
    /// Envelope for interpolation of amplitude.
    pub amplitude: AmpEnvelope,
    /// Envelope for interpolation of frequency.
    pub frequency: FreqEnvelope,
}

impl Oscillator {

    /// Oscillator constructor.
    #[inline]
    pub fn new() -> Oscillator {
        Oscillator {
            waveform: Waveform::Sine,
            phase: 0.0,
            amplitude: Envelope::zeroed(),
            frequency: Envelope::zeroed(),
            gaussian_perc: 0.0,
        }
    }

    /// Waveform builder method.
    #[inline]
    pub fn waveform(self, waveform: Waveform) -> Oscillator {
        Oscillator { waveform: waveform, ..self }
    }

    /// Amplitude envelope builder method.
    #[inline]
    pub fn amplitude(self, amp_env: AmpEnvelope) -> Oscillator {
        Oscillator { amplitude: amp_env, ..self }
    }

    /// Amplitude envelope builder method.
    #[inline]
    pub fn frequency(self, freq_env: FreqEnvelope) -> Oscillator {
        Oscillator { frequency: freq_env, ..self }
    }

    /// Set a gaussian randomness to the frequency envelope value retrieval
    /// for a "warbly" effect.
    #[inline]
    pub fn warbliness(self, warbliness: f32) -> Oscillator {
        Oscillator { gaussian_perc: warbliness, ..self }
    }

    /// Calculate and return the amplitude at the given ratio.
    #[inline]
    pub fn amp_at_ratio(&mut self, ratio: f64, note_freq_multi: f64, sample_hz: f64) -> f32 {
        let phase = self.phase;
        // Determine the overall frequency and in turn how much to advance the phase.
        let freq_at_ratio = self.freq_at_ratio(ratio) * note_freq_multi;
        self.phase = self.waveform.next_phase(phase, freq_at_ratio, sample_hz);
        self.waveform.amp_at_phase(phase) * self.amplitude.y(ratio) as f32
    }

    /// Calculate and return the frequency at
    /// the given ratio.
    #[inline]
    pub fn freq_at_ratio(&self, ratio: f64) -> f64 {
        use gaussian;
        let freq = self.frequency.y(ratio);
        let hz = match self.gaussian_perc > 0.0 {
            true => gaussian::gen(0.5f64, self.gaussian_perc)
                    * (freq / 0.5)
                    * (MAX_HZ as f64 - MIN_HZ as f64)
                    + MIN_HZ as f64,
            false => freq * ((MAX_HZ - MIN_HZ) + MIN_HZ) as f64
        };
        hz
    }

}

