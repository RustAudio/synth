//! 
//! Synthesis Oscillator module.
//!

pub use self::waveform::Waveform;
pub use self::amplitude::Amplitude;
pub use self::amplitude::Envelope as AmpEnvelope;
pub use self::frequency::Frequency;
pub use self::frequency::Envelope as FreqEnvelope;
pub use self::freq_warp::FreqWarp;

use time;

pub mod waveform;
pub mod amplitude;
pub mod frequency;
pub mod freq_warp;


/// The fundamental component of a synthesizer.
#[derive(Debug, Clone, PartialEq)]
pub struct Oscillator<W, A, F, FW> {
    /// Waveform used for phase movement.
    pub waveform: W,
    /// Envelope for amplitude interpolation.
    pub amplitude: A,
    /// Envelope for frequency interpolation.
    pub frequency: F,
    /// A type used for warping the Oscillator's frequency.
    pub freq_warp: FW,
    /// Whether or not the Oscillator is currently muted.
    pub is_muted: bool,
}

/// The state of an Oscillator that is unique to each voice playing it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// The Oscillator's current phase.
    pub phase: f64,
    /// The phase of the FreqWarp used to warp the oscillator's frequency.
    pub freq_warp_phase: f64,
}

/// The state of each oscillator per-voice.
#[derive(Clone, Debug, PartialEq)]
pub struct StatePerVoice(pub Vec<State>);


impl State {
    pub fn new() -> Self {
        State {
            phase: 0.0,
            freq_warp_phase: 0.0,
        }
    }
}

impl<W, A, F, FW> Oscillator<W, A, F, FW> {

    /// Oscillator constructor.
    #[inline]
    pub fn new(waveform: W, amplitude: A, frequency: F, freq_warp: FW) -> Self {
        Oscillator {
            waveform: waveform,
            amplitude: amplitude,
            frequency: frequency,
            freq_warp: freq_warp,
            is_muted: false,
        }
    }

    /// Waveform builder method.
    #[inline]
    pub fn waveform(mut self, waveform: W) -> Self {
        self.waveform = waveform;
        self
    }

    /// Amplitude envelope builder method.
    #[inline]
    pub fn amplitude(mut self, amplitude: A) -> Self {
        self.amplitude = amplitude;
        self
    }

    /// Amplitude envelope builder method.
    #[inline]
    pub fn frequency(mut self, frequency: F) -> Self {
        self.frequency = frequency;
        self
    }

    /// Calculate and return the amplitude at the given ratio.
    #[inline]
    pub fn amp_at(&self, phase: f64, playhead_perc: f64) -> f32 where
        A: Amplitude,
        W: Waveform,
    {
        self.waveform.amp_at_phase(phase) * self.amplitude.amp_at_playhead(playhead_perc)
    }

    /// Calculate and return the phase that should follow some given phase.
    #[inline]
    pub fn next_frame_phase(&self,
                            sample_hz: f64,
                            playhead_perc: f64,
                            note_freq_multi: f64,
                            phase: f64,
                            freq_warp_phase: &mut f64) -> f64
        where W: Waveform,
              F: Frequency,
              FW: FreqWarp,
    {
        let hz = self.frequency.hz_at_playhead(playhead_perc);
        let hz = self.waveform.process_hz(hz);
        self.freq_warp.step_phase(sample_hz, freq_warp_phase);
        let warped_hz = self.freq_warp.warp_hz(hz, *freq_warp_phase);
        let note_hz = warped_hz * note_freq_multi;
        phase + (note_hz / sample_hz)
    }

    /// Steps forward the given `phase` and `freq_warp_phase` and yields the amplitude for the
    /// next frame.
    #[inline]
    pub fn next_frame_amp(&mut self,
                          sample_hz: time::SampleHz,
                          playhead_perc: f64,
                          note_freq_multi: f64,
                          state: &mut State) -> f32
        where A: Amplitude,
              W: Waveform,
              F: Frequency,
              FW: FreqWarp,
    {
        let amp = self.amp_at(state.phase, playhead_perc);
        let next_phase = self.next_frame_phase(sample_hz,
                                               playhead_perc,
                                               note_freq_multi,
                                               state.phase,
                                               &mut state.freq_warp_phase);
        state.phase = next_phase;
        amp
    }

}
