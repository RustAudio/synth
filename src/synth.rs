//!
//!  synth.rs
//!
//!  Created by Mitchell Nordine at 03:37PM on July 02, 2014.
//!
//!

//! Implementation of the `Synth` struct for basic multi-voice, multi-oscillator envelope synthesis.

use dsp::Node as DspNode;
use dsp::Settings as DspSettings;
use dsp::Sample;
use oscillator::Oscillator;
use pitch;
use std::iter::repeat;
use time::{self, Ms};
use voice::{Voice, NoteHz, NoteState, NoteVelocity};

pub type Duration = time::calc::Ms;
pub type BasePitch = pitch::calc::Hz;
pub type LoopStart = f64;
pub type LoopEnd = f64;
pub type Attack = time::calc::Ms;
pub type Release = time::calc::Ms;
pub type Playhead = time::calc::Samples;

/// The mode in which the Synth will handle notes.
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub enum Mode {
    /// Single voice.
    Mono(Voice, Mono),
    /// Multiple voices.
    Poly(Vec<Voice>),
}

/// The state of monophony.
#[derive(Copy, Clone, Debug, RustcDecodable, RustcEncodable)]
pub enum Mono {
    /// New notes will reset the voice's playheads
    Normal,
    /// New notes will not reset the voice's playheads.
    Legato,
}

/// The `Synth` generates audio via a vector of `Voice`s,
/// while a `Voice` generates audio via a vector of
/// `Oscillator`s, creating a small DSP tree.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Synth {
    /// Oscillators for playback.
    pub oscillators: Vec<Oscillator>,
    /// The mode of note playback.
    pub mode: Mode,
    /// Duration of the Synth instrument in samples.
    pub duration: Duration,
    /// Base pitch of the Synth instrument in Steps.
    pub base_pitch: BasePitch,
    /// Amplitude multiplier (volume).
    pub vol: f32,
    /// Normaliser for the Synth.
    pub normaliser: f32,
    /// Data used for looping over a duration of the Synth.
    pub loop_data: Option<(LoopStart, LoopEnd)>,
    /// Data used for fading in / out from playback.
    pub fade_data: Option<(Attack, Release)>,
    /// Is the playback currently paused?
    pub is_paused: bool,
}

const MS_300: Duration = 300.0;
const C_1: BasePitch = 32.703;


impl Synth {

    /// Constructor for a new Synth.
    #[inline]
    pub fn new() -> Synth {
        Synth {
            oscillators: vec![Oscillator::new()],
            mode: Mode::Mono(Voice::new(1), Mono::Normal),
            duration: MS_300,
            base_pitch: C_1,
            vol: 1.0,
            normaliser: 1.0,
            loop_data: None,
            fade_data: None,
            is_paused: false,
        }
    }

    /// Set the number of voices that the Synth shall use.
    /// If there are no voices, a default voice will be constructed.
    #[inline]
    pub fn num_voices(mut self, num_voices: usize) -> Synth {
        if num_voices == 0 {
            println!("A Synth must have at least one voice, but the requested number is 0.");
            self
        } else if num_voices == 1 {
            let voice = match self.mode {
                Mode::Mono(_, _) => return self,
                Mode::Poly(ref mut voices) => voices.swap_remove(0),
            };
            Synth { mode: Mode::Mono(voice, Mono::Normal), ..self }
        } else {
            let mut voices = match self.mode {
                Mode::Mono(ref voice, _) => vec![voice.clone()],
                Mode::Poly(voices) => voices,
            };
            let len = voices.len();
            if num_voices == len {
                Synth { mode: Mode::Poly(voices), ..self }
            } else if len < num_voices {
                let last_voice = voices[len-1].clone();
                voices.extend(repeat(last_voice).take(num_voices - len));
                Synth { mode: Mode::Poly(voices), ..self }
            } else {
                let voices = voices.into_iter().take(num_voices).collect();
                Synth { mode: Mode::Poly(voices), ..self }
            }
        }
    }

    /// Turn legato on or off. If the Mode was originally Poly and legato was turned on, the Mode
    /// will become Mono(_, Legato).
    pub fn legato(mut self, on: bool) -> Synth {
        if on {
            self = self.num_voices(1);
            if let Mode::Mono(_, ref mut mono) = self.mode {
                *mono = Mono::Legato;
            }
        }
        self
    }

    /// Add an oscillator to a Synth.
    #[inline]
    pub fn oscillator(mut self, oscillator: Oscillator) -> Synth {
        self.oscillators.push(oscillator);
        match self.mode {
            Mode::Mono(ref mut voice, _) => voice.oscillator_phases.push(0.0),
            Mode::Poly(ref mut voices) => for voice in voices.iter_mut() {
                voice.oscillator_phases.push(0.0);
            },
        }
        self
    }

    /// Add multiple oscillators to a Synth.
    #[inline]
    pub fn oscillators<I: Iterator<Item=Oscillator>>(mut self, oscillators: I) -> Synth {
        let len = self.oscillators.len();
        self.oscillators.extend(oscillators);
        let target_len = self.oscillators.len();
        let add_phases = |voice: &mut Voice| voice.oscillator_phases
            .extend((len..target_len).map(|_| 0.0));
        match self.mode {
            Mode::Mono(ref mut voice, _) => add_phases(voice),
            Mode::Poly(ref mut voices)   => for voice in voices.iter_mut() { add_phases(voice) },
        }
        self
    }

    /// Set the Synth's duration.
    #[inline]
    pub fn duration(self, duration: Duration) -> Synth {
        Synth { duration: duration, ..self }
    }

    /// Set the Synth's base pitch.
    #[inline]
    pub fn base_pitch(self, base_pitch: BasePitch) -> Synth {
        Synth { base_pitch: base_pitch, ..self }
    }

    /// Set the Synth's volume.
    #[inline]
    pub fn volume(self, vol: f32) -> Synth {
        Synth { vol: vol, ..self }
    }

    /// Set the Synth's normaliser.
    #[inline]
    pub fn normaliser(self, normaliser: f32) -> Synth {
        Synth { normaliser: normaliser, ..self }
    }

    /// Set the loop data for the synth.
    #[inline]
    pub fn loop_points(self, start: LoopStart, end: LoopEnd) -> Synth {
        Synth { loop_data: Some((start, end)), ..self }
    }

    /// Set the fade data for the synth.
    #[inline]
    pub fn fade(self, attack: Attack, release: Release) -> Synth {
        Synth { fade_data: Some((attack, release)), ..self }
    }

    /// Set the start loop point.
    #[inline]
    pub fn loop_start(self, start: LoopStart) -> Synth {
        let loop_data = match self.loop_data {
            Some((_, end)) => Some((start, end)),
            None => Some((start, 1.0))
        };
        Synth { loop_data: loop_data, ..self }
    }

    /// Set the end loop point.
    #[inline]
    pub fn loop_end(self, end: LoopEnd) -> Synth {
        let loop_data = match self.loop_data {
            Some((start, _)) => Some((start, end)),
            None => Some((0.0, end))
        };
        Synth { loop_data: loop_data, ..self }
    }

    /// Set the attack.
    #[inline]
    pub fn attack(self, attack: Attack) -> Synth {
        let fade_data = match self.fade_data {
            Some((_, release)) => Some((attack, release)),
            None => Some((attack, 0.0))
        };
        Synth { fade_data: fade_data, ..self }
    }

    /// Set the release.
    #[inline]
    pub fn release(self, release: Release) -> Synth {
        let fade_data = match self.fade_data {
            Some((attack, _)) => Some((attack, release)),
            None => Some((0.0, release))
        };
        Synth { fade_data: fade_data, ..self }
    }

    /// Add an oscillator.
    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        match self.mode {
            Mode::Mono(ref mut voice, _) => voice.oscillator_phases.push(0.0),
            Mode::Poly(ref mut voices) => for voice in voices.iter_mut() {
                voice.oscillator_phases.push(0.0);
            }
        }
    }

    /// Remove and return the oscillator at the given idx.
    pub fn remove_oscillator(&mut self, idx: usize) -> Oscillator {
        match self.mode {
            Mode::Mono(ref mut voice, _) => { voice.oscillator_phases.remove(idx); },
            Mode::Poly(ref mut voices) => for voice in voices.iter_mut() {
                voice.oscillator_phases.remove(idx);
            },
        }
        self.oscillators.remove(idx)
    }

    /// Return whether or not there are any currently active voices.
    #[inline]
    pub fn is_active(&self) -> bool {
        if self.is_paused { return false }
        match self.mode {
            Mode::Mono(ref voice, _) => voice.maybe_note.is_some(),
            Mode::Poly(ref voices) => voices.iter().any(|voice| voice.maybe_note.is_some()),
        }
    }

    /// Begin playback of a note. Synth will try to use a free `Voice` to do this.
    /// If no `Voice`s are free, the one playing the oldest note will be chosen to
    /// play the new note instead.
    #[inline]
    pub fn note_on(&mut self, note_hz: NoteHz, note_velocity: NoteVelocity) {
        let note_freq_multi = note_hz as f64 / self.base_pitch as f64;
        match self.mode {
            Mode::Mono(ref mut voice, mono) => {
                if let Mono::Normal = mono {
                    voice.reset_playheads();
                }
                voice.note_on(note_hz, note_freq_multi, note_velocity);
            },
            Mode::Poly(ref mut voices) => {
                let mut oldest: Option<&mut Voice> = None;
                let mut max_sample_count: i64 = 0;
                for voice in voices.iter_mut() {
                    if voice.maybe_note.is_none() {
                        voice.reset_playheads();
                        voice.note_on(note_hz, note_freq_multi, note_velocity);
                        return;
                    }
                    else if voice.playhead >= max_sample_count {
                        max_sample_count = voice.playhead;
                        oldest = Some(voice);
                    }
                }
                if let Some(voice) = oldest {
                    voice.reset_playheads();
                    voice.note_on(note_hz, note_freq_multi, note_velocity);
                }
            }
        }
    }

    /// Stop playback of the note that was triggered with the matching frequency.
    #[inline]
    pub fn note_off(&mut self, note_hz: NoteHz) {
        let is_match = |voice: &Voice| match voice.maybe_note {
            Some((NoteState::Playing, voice_note_hz, _, _)) => voice_note_hz == note_hz,
            _ => false,
        };
        match self.mode {
            Mode::Mono(ref mut voice, _) => if is_match(voice) { voice.note_off() },
            Mode::Poly(ref mut voices) => {
                let maybe_voice = voices.iter_mut().fold(None, |maybe_current_match, voice| {
                    if is_match(voice) {
                        match maybe_current_match {
                            None => return Some(voice),
                            Some(ref current_match) => if voice.playhead >= current_match.playhead {
                                return Some(voice)
                            },
                        }
                    }
                    maybe_current_match
                });
                if let Some(voice) = maybe_voice {
                    voice.note_off();
                }
            },
        }
    }

    /// Pause playback.
    #[inline]
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Unpause playback.
    #[inline]
    pub fn unpause(&mut self ) {
        self.is_paused = false;
    }

    /// Stop playback and clear the current notes.
    #[inline]
    pub fn stop(&mut self) {
        match self.mode {
            Mode::Mono(ref mut voice, _) => voice.stop(),
            Mode::Poly(ref mut voices) => for voice in voices.iter_mut() { voice.stop() },
        }
    }

}

impl<S> DspNode<S> for Synth where S: Sample {

    #[inline]
    fn audio_requested(&mut self, output: &mut [S], settings: DspSettings) {
        if !self.is_active() { return }
        let sample_hz = settings.sample_hz as f64;
        let Synth {
            ref oscillators,
            ref mut mode,
            duration,
            vol,
            normaliser,
            ref loop_data,
            ref fade_data,
            ..
        } = *self;

        // Convert the duration to samples.
        let duration = Ms(duration).samples(sample_hz);

        // Convert the loop points from duration percentages to samples.
        let loop_data_samples = loop_data.map(|(start_perc, end_perc)| {
            use num::Float;
            ((start_perc * duration as f64).round() as time::calc::Samples,
             (end_perc * duration as f64).round() as time::calc::Samples)
        });

        // Convert the fade durations from milliseconds to samples.
        let fade_data_samples = fade_data.map(|(attack_ms, release_ms)| {
            (Ms(attack_ms).samples(sample_hz), Ms(release_ms).samples(sample_hz))
        });

        let amp_multi = vol * normaliser;
        let vol_per_channel = vec![amp_multi; settings.channels as usize];

        // Request audio from a voice and sum it to the output.
        let request_audio = |output: &mut [S], voice: &mut Voice| {
            let mut working: Vec<S> = vec![Sample::zero(); settings.buffer_size()];
            voice.fill_buffer(&mut working,
                              settings,
                              oscillators,
                              duration,
                              loop_data_samples.as_ref(),
                              fade_data_samples.as_ref());
            Sample::add_buffers(output, &working, &vol_per_channel);
        };

        // Request audio from each voice.
        match *mode {
            Mode::Mono(ref mut voice, _) => request_audio(output, voice),
            Mode::Poly(ref mut voices) => for voice in voices.iter_mut() {
                request_audio(output, voice);
            }
        }
    }

}

