//!
//!  synth_voice.rs
//!
//!  Created by Mitchell Nordine at 04:01PM on June 28, 2014.
//!
//!

use dsp::Settings as DspSettings;
use dsp::{Sample};
use oscillator::Oscillator;
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
pub struct Voice {
    /// The current phase for each oscillator owned by the Synth.
    pub oscillator_phases: Vec<f64>,
    /// Data for a note, if there is one currently being played.
    pub maybe_note: Option<(NoteState, NoteHz, CurrentFreq, NoteVelocity)>,
    /// Playhead over the current note.
    pub playhead: Playhead,
    /// Playhead over the loop duration.
    pub loop_playhead: LoopPlayhead,
}

/// The current state of the Voice's note playback.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum NoteState {
    /// The note is current playing.
    Playing,
    /// The note has been released and is fading out.
    Released(Playhead),
}

/// The Voice's currently playing frequency.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum CurrentFreq {
    /// The frequency is currently sliding due to some portamento.
    Portamento(time::calc::Ms, pitch::calc::Mel, time::calc::Ms, pitch::calc::Mel),
    /// We have a constant frequency and the note multiplier has already been calculated.
    Constant(NoteHz),
}


impl CurrentFreq {

    /// Construct a new CurrentFreq considering portamento and detuning.
    pub fn new(portamento: f64,
               detune: f32,
               note_hz: NoteHz,
               maybe_last_hz: Option<pitch::calc::Hz>) -> CurrentFreq {

        // If some detune was given, slightly detune the note_hz.
        let target_hz = if detune > 0.0 {
            let step_offset = ::rand::random::<f32>() * 2.0 * detune - detune;
            pitch::Step(Hz(note_hz).step() + step_offset).hz()
        // Otherwise, our target_hz is the given note_hz.
        } else {
            note_hz
        };

        match (portamento > 0.0, maybe_last_hz) {
            // If we have some portamento and a note to slide from, create a Portamento.
            (true, Some(hz)) =>
                CurrentFreq::Portamento(0.0, Hz(hz).mel(), portamento, Hz(target_hz).mel()),
            // Otherwise, we have a constant frequency.
            _ => CurrentFreq::Constant(target_hz),
        }
    }

    /// Calculate the hz given the state of the CurrentFreq.
    pub fn hz(&self) -> pitch::calc::Hz {
        match *self {
            CurrentFreq::Portamento(count_ms, start_mel, duration_ms, target_mel) => {
                let perc = count_ms as f64 / duration_ms as f64;
                let diff_mel = target_mel - start_mel;
                let perc_diff_mel = perc * diff_mel as f64;
                let mel = start_mel + perc_diff_mel as pitch::calc::Mel;
                pitch::Mel(mel).hz()
            },
            CurrentFreq::Constant(note_hz) => note_hz,
        }
    }

}


impl Voice {

    /// Constructor for a Voice.
    pub fn new(num_oscillators: usize) -> Voice {
        Voice {
            oscillator_phases: (0..num_oscillators).map(|_| 0.0).collect(),
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
    pub fn note_on(&mut self, hz: NoteHz, freq: CurrentFreq, vel: NoteVelocity) {
        self.maybe_note = Some((NoteState::Playing, hz, freq, vel));
    }

    /// Release playback of the current not eif there is one.
    #[inline]
    pub fn note_off(&mut self) {
        self.maybe_note = self.maybe_note.map(|(_, h, m, v)| (NoteState::Released(0), h, m, v));
    }

    /// Stop playback of the current note if there is one and reset the playheads.
    #[inline]
    pub fn stop(&mut self) {
        for phase in self.oscillator_phases.iter_mut() { *phase = 0.0; }
        self.maybe_note = None;
        self.playhead = 0;
        self.loop_playhead = 0;
    }

    /// Generate and fill the audio buffer for the given parameters.
    #[inline]
    pub fn fill_buffer<S>(&mut self,
                          output: &mut [S],
                          settings: DspSettings,
                          oscillators: &[Oscillator],
                          duration: time::calc::Samples,
                          base_pitch: pitch::calc::Hz,
                          loop_data: Option<&(LoopStart, LoopEnd)>,
                          fade_data: Option<&(Attack, Release)>)
        where S: Sample
    {
        let Voice {
            ref mut oscillator_phases,
            ref mut playhead,
            ref mut loop_playhead,
            ref mut maybe_note,
        } = *self;

        let (attack, release) = fade_data.map_or_else(|| (0, 0), |&(a, r)| (a, r));
        let velocity = maybe_note.map_or_else(|| 1.0, |(_, _, _, velocity)| velocity);

        let frame_ms = Samples(1).ms(settings.sample_hz as f64);

        for frame in output.chunks_mut(settings.channels as usize) {

            // Calculate the amplitude of the current frame.
            let wave = if maybe_note.is_some() && *loop_playhead < duration {
                let ratio = *loop_playhead as f64 / duration as f64;

                let (note_state, current_freq) = maybe_note.as_mut()
                    .map(|&mut(note_state, _, ref mut freq, _)| {
                        let current_freq = *freq;
                        // Update the ms count of the portamento.
                        let maybe_constant_hz = match *freq {
                            CurrentFreq::Portamento(ref mut ms, _, dur, target_mel) => {
                                *ms = *ms + frame_ms;
                                if *ms > dur { Some(pitch::Mel(target_mel).hz()) } else { None }
                            },
                            _ => None,
                        };
                        // If the ms count has exceeded the duration, set the freq.
                        if let Some(constant_hz) = maybe_constant_hz {
                            *freq = CurrentFreq::Constant(constant_hz);
                        }
                        (note_state, current_freq)
                    }).unwrap();

                let freq_multi = current_freq.hz() as f64 / base_pitch as f64;

                // Sum the amplitude of each oscillator at the given ratio.
                let active_oscillators = oscillators.iter().zip(oscillator_phases.iter_mut())
                    .filter(|&(osc, _)| !osc.is_muted);
                active_oscillators.fold(0.0, |total, (osc, phase)| {
                    let mut wave = osc.amp_at_ratio(*phase, ratio);
                    *phase = osc.next_phase(*phase, ratio, freq_multi, settings.sample_hz as f64);
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

