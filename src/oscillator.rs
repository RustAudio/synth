
//! Synthesis Oscillator module.

use env_point::Point;
use pitch;
use envelope::Envelope;
use waveform::Waveform;

pub type AmpEnvelope = Envelope<f64, f64, Point>;
pub type FreqEnvelope = Envelope<f64, f64, Point>;

/// The fundamental component of a synthesizer.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Oscillator {
    /// Waveform used for phase movement.
    pub waveform: Waveform,
    /// The percentage of randomness to be applied to freq.
    pub gaussian_perc: f32,
    /// Envelope for interpolation of amplitude.
    pub amplitude: AmpEnvelope,
    /// Envelope for interpolation of frequency.
    pub frequency: FreqEnvelope,
    /// Whether or not the Oscillator is currently muted.
    pub is_muted: bool,
}

impl Oscillator {

    /// Oscillator constructor.
    #[inline]
    pub fn new() -> Oscillator {
        Oscillator {
            waveform: Waveform::Sine,
            amplitude: Envelope::from_points(vec![Point::new(0.0, 0.0, 0.0),
                                                  Point::new(1.0, 0.0, 0.0)]),
            frequency: Envelope::from_points(vec![Point::new(0.0, 0.0, 0.0),
                                                  Point::new(1.0, 0.0, 0.0)]),
            gaussian_perc: 0.0,
            is_muted: false,
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
    pub fn amp_at_ratio(&self, phase: f64, ratio: f64) -> f32 {
        let env_amplitude = match self.amplitude.y(ratio) {
            Some(y) => y as f32,
            None => panic!("The given ratio {:?} is out of range of the Envelope's X axis.", ratio),
        };
        self.waveform.amp_at_phase(phase) * env_amplitude
    }

    /// Calculate and return the next phase for the given phase.
    #[inline]
    pub fn next_phase(&self, phase: f64, ratio: f64, note_freq_multi: f64, sample_hz: f64) -> f64 {
        let freq_at_ratio = self.freq_at_ratio(ratio) * note_freq_multi;
        phase + (freq_at_ratio / sample_hz)
    }

    /// Calculate and return the frequency at
    /// the given ratio.
    #[inline]
    pub fn freq_at_ratio(&self, ratio: f64) -> f64 {
        use gaussian;
        let mut freq = match self.frequency.y(ratio) {
            Some(y) => y,
            None => panic!("The given ratio is out of range of the Envelope's X axis."),
        };
        if self.waveform == Waveform::NoiseWalk {
            freq = pitch::ScaledPerc(freq, 0.6).perc()
        }
        let hz = if self.gaussian_perc > 0.0 {
            use num::Float;
            let mels = pitch::Perc(freq).mel();
            let gaus_mels = mels
                          + gaussian::gen(0.5f32, self.gaussian_perc.powf(2.0))
                          * 1000.0
                          - 500.0;
            pitch::Mel(gaus_mels).hz()
        } else {
            pitch::Perc(freq).hz()
        } as f64;
        hz
    }

}

