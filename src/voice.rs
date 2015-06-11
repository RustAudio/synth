//!
//!  synth_voice.rs
//!
//!  Created by Mitchell Nordine at 04:01PM on June 28, 2014.
//!
//!

use dsp::Settings as DspSettings;
use dsp::{Sample};
use oscillator::{Amplitude, Frequency, FreqWarp, Oscillator, Waveform};
use note_freq::NoteFreq;
use pitch::{self, Hz};
use time::{self, Samples};


pub type Playhead = time::calc::Samples;
pub type LoopStart = time::calc::Samples;
pub type LoopEnd = time::calc::Samples;
pub type Attack = time::calc::Samples;
pub type Release = time::calc::Samples;
pub type LoopPlayhead = time::calc::Samples;
pub type NoteDuration = time::calc::Samples;
pub type NoteFreqMulti = f64;
pub type NoteHz = f32;
pub type NoteVelocity = f32;

/// A single Voice. A Synth may consist
/// of any number of Voices.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Voice<NF> {
    /// The current phase for each oscillator owned by the Synth.
    pub oscillator_states: Vec<OscillatorState>,
    /// Data for a note, if there is one currently being played.
    pub maybe_note: Option<(NoteState, NoteHz, NF, NoteVelocity)>,
    /// Playhead over the current note.
    pub playhead: Playhead,
    /// Playhead over the loop duration.
    pub loop_playhead: LoopPlayhead,
}

/// The state of an Oscillator that is unique to each voice playing it.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct OscillatorState {
    /// The Oscillator's current phase.
    pub phase: f64,
    /// The phase of the FreqWarp used to warp the oscillator's frequency.
    pub freq_warp_phase: f64,
}

/// The current state of the Voice's note playback.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum NoteState {
    /// The note is current playing.
    Playing,
    /// The note has been released and is fading out.
    Released(Playhead),
}


impl OscillatorState {
    pub fn new() -> OscillatorState {
        OscillatorState {
            phase: 0.0,
            freq_warp_phase: 0.0,
        }
    }
}


impl<NF> Voice<NF> {

    /// Constructor for a Voice.
    pub fn new(num_oscillators: usize) -> Voice<NF> {
        Voice {
            oscillator_states: (0..num_oscillators).map(|_| OscillatorState {
                phase: 0.0,
                freq_warp_phase: 0.0,
            }).collect(),
            maybe_note: None,
            playhead: 0,
            loop_playhead: 0,
        }
    }

    /// Reset the voice's playheads.
    #[inline]
    pub fn reset_playheads(&mut self) {
        self.playhead = 0;
        self.loop_playhead = 0;
    }

    /// Trigger playback with the given note, resetting all playheads.
    #[inline]
    pub fn note_on(&mut self, hz: NoteHz, freq: NF, vel: NoteVelocity) {
        self.maybe_note = Some((NoteState::Playing, hz, freq, vel));
    }

    /// Release playback of the current not eif there is one.
    #[inline]
    pub fn note_off(&mut self) {
        if let Some(&mut(ref mut state, _, _, _)) = self.maybe_note.as_mut() {
            *state = NoteState::Released(0);
        }
    }

    /// Stop playback of the current note if there is one and reset the playheads.
    #[inline]
    pub fn stop(&mut self) {
        for osc_state in self.oscillator_states.iter_mut() {
            osc_state.phase = 0.0;
            osc_state.freq_warp_phase = 0.0;
        }
        self.maybe_note = None;
        self.playhead = 0;
        self.loop_playhead = 0;
    }

    /// Generate and fill the audio buffer for the given parameters.
    #[inline]
    pub fn fill_buffer<S, W, A, F, FW>(&mut self,
                                       output: &mut [S],
                                       settings: DspSettings,
                                       oscillators: &[Oscillator<W, A, F, FW>],
                                       duration: time::calc::Samples,
                                       base_pitch: pitch::calc::Hz,
                                       loop_data: Option<&(LoopStart, LoopEnd)>,
                                       fade_data: Option<&(Attack, Release)>) where
        NF: NoteFreq,
        S: Sample,
        W: Waveform,
        A: Amplitude,
        F: Frequency,
        FW: FreqWarp,
    {
        let Voice {
            ref mut oscillator_states,
            ref mut playhead,
            ref mut loop_playhead,
            ref mut maybe_note,
        } = *self;

        let DspSettings { sample_hz, channels, .. } = settings;
        let (attack, release) = fade_data.map_or_else(|| (0, 0), |&(a, r)| (a, r));
        let velocity = maybe_note.as_ref().map_or_else(|| 1.0, |&(_, _, _, velocity)| velocity);

        let frame_ms = Samples(1).ms(sample_hz as f64);

        for frame in output.chunks_mut(channels as usize) {

            // Calculate the amplitude of the current frame.
            let wave = if maybe_note.is_some() && *loop_playhead < duration {
                let playhead_perc = *loop_playhead as f64 / duration as f64;

                let (note_state, hz) = maybe_note.as_mut()
                    .map(|&mut(note_state, _, ref mut freq, _)| {
                        freq.step_frame(frame_ms);
                        (note_state, freq.hz())
                    }).unwrap();

                let freq_multi = hz as f64 / base_pitch as f64;

                // Sum the amplitude of each oscillator at the given ratio.
                let active_oscillators = oscillators.iter().zip(oscillator_states.iter_mut())
                    .filter(|&(osc, _)| !osc.is_muted);

                // Fold the active oscillators and their state
                active_oscillators.fold(0.0, |total, (osc, osc_state)| {

                    // Determine the amplitude at the current phase and playhead position.
                    let mut wave = osc.amp_at(osc_state.phase, playhead_perc);

                    // Update the oscillator state's phase and freq_warp_phase.
                    osc_state.phase = osc.next_phase(osc_state.phase,
                                                     playhead_perc,
                                                     freq_multi,
                                                     sample_hz as f64,
                                                     &mut osc_state.freq_warp_phase);

                    // If within the attack duration, apply the fade.
                    if *playhead < attack {
                        wave *= *playhead as f32 / attack as f32;
                    }

                    // If within the release duration, apply the fade.
                    if let NoteState::Released(release_playhead) = note_state {
                        wave *= (release - release_playhead) as f32 / release as f32;
                    }

                    wave + total
                })
            } else {
                // If the playhead is out of range or if there is no note, zero the frame.
                0.0
            };

            // Assign the amp to each channel.
            for channel in frame.iter_mut() {
                *channel = Sample::from_wave(wave * velocity);
            }

            // Iterate the release playhead and check for whether or not the release playhead
            // exceeds the release limit. If it does, reset the note.
            let note_should_reset = match *maybe_note {
                Some((NoteState::Released(ref mut release_playhead), _, _, _)) => {
                    *release_playhead += 1;
                    *release_playhead > release
                },
                None => continue,
                _ => false,
            };
            if note_should_reset {
                *maybe_note = None;
                *playhead = 0;
            }

            // Iterate the loop_playhead. If the loop_playhead passes the loop_end,
            // reset the playhead to the start.
            *loop_playhead += 1;
            if let Some(&(loop_start, loop_end)) = loop_data {
                if *loop_playhead >= loop_end {
                    *loop_playhead = (*loop_playhead - loop_end) + loop_start;
                }
            }

            // Iterate the playhead. If the playhead passes the duration of the instrument or
            // the note that is currently being played, reset the playhead and stop playback.
            *playhead += 1;
            if *loop_playhead > duration {
                *maybe_note = None;
                *playhead = 0;
            }

        }

    }

}

