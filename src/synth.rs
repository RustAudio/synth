//!
//!  synth.rs
//!
//!  Created by Mitchell Nordine at 03:37PM on July 02, 2014.
//!
//!
//!  Implementation of the `Synth` struct for basic multi-voice, multi-oscillator envelope synthesis.
//!

use dsp::Node as DspNode;
use dsp::Settings as DspSettings;
use dsp::Sample;
use oscillator::Oscillator;
use panning::stereo;
use pitch;
use std::iter::repeat;
use time::{self, Ms};
use voice::{CurrentFreq, NoteHz, NoteState, NoteVelocity, Voice};


pub type Duration = time::calc::Ms;
pub type BasePitch = pitch::calc::Hz;
pub type LoopStart = f64;
pub type LoopEnd = f64;
pub type Attack = time::calc::Ms;
pub type Release = time::calc::Ms;
pub type Playhead = time::calc::Samples;

/// The `Synth` generates audio via a vector of `Voice`s,
/// while a `Voice` generates audio via a vector of
/// `Oscillator`s, creating a small DSP tree.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Synth {
    /// Oscillators for playback.
    pub oscillators: Vec<Oscillator>,
    /// The mode of note playback.
    pub mode: Mode,
    /// The amplitude for each channel.
    pub channels: Vec<f32>,
    /// The voices used by the Synth.
    /// - If the Synth is in Poly mode, it will play one voice at a time.
    /// - If the Synth is in Mono mode, it will play all voices at once.
    pub voices: Vec<Voice>,
    /// The amount each voice's note_on should be detuned.
    pub detune: f32,
    /// The amount each voice should be spread across the available channels.
    pub spread: f32,
    /// Duration of the Synth instrument in samples.
    pub duration: Duration,
    /// Base pitch of the Synth instrument in Steps.
    pub base_pitch: BasePitch,
    /// Amplitude multiplier (volume).
    pub volume: f32,
    /// The duration at which it takes to drift from the current note to some new note.
    pub portamento: f64,
    /// Data used for looping over a duration of the Synth.
    pub loop_data: Option<(LoopStart, LoopEnd)>,
    /// Data used for fading in / out from playback.
    pub fade_data: Option<(Attack, Release)>,
    /// Is the playback currently paused?
    pub is_paused: bool,
}

/// The mode in which the Synth will handle notes.
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub enum Mode {
    /// Single voice (normal or legato) with a stack of fallback notes.
    Mono(Mono, Vec<NoteHz>),
    /// Multiple voices.
    Poly,
}

/// The state of monophony.
#[derive(Copy, Clone, Debug, RustcDecodable, RustcEncodable)]
pub enum Mono {
    /// New notes will reset the voice's playheads
    Normal,
    /// If a note is already playing, new notes will not reset the voice's playheads.
    /// A stack of notes is kept - if a NoteOff occurs on the current note, it is replaced with the
    /// note at the top of the stack if there is one. The stacked notes are reset if the voice
    /// becomes inactive.
    Legato,
}


/// Construct an empty note stack for the Mono synth mode.
pub fn empty_note_stack() -> Vec<NoteHz> {
    Vec::with_capacity(16)
}


impl Synth {

    /// Constructor for a new Synth.
    #[inline]
    pub fn new() -> Synth {
        const MS_300: Duration = 300.0;
        const C_1: BasePitch = 32.703;
        Synth {
            oscillators: Vec::new(),
            mode: Mode::Mono(Mono::Normal, empty_note_stack()),
            channels: Vec::from(&stereo::centre()[..]),
            voices: vec![Voice::new(0)],
            detune: 0.0,
            spread: 0.0,
            duration: MS_300,
            base_pitch: C_1,
            volume: 1.0,
            portamento: 0.0,
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
        } else {
            let len = self.voices.len();
            if num_voices == len {
                self
            } else if len < num_voices {
                let last_voice = self.voices[len-1].clone();
                self.voices.extend(repeat(last_voice).take(num_voices - len));
                self
            } else {
                self.voices.truncate(num_voices);
                self
            }
        }
    }

    /// Turn legato on or off. If the Mode was originally Poly and legato was turned on, the Mode
    /// will become Mono(_, Legato).
    pub fn legato(mut self, on: bool) -> Synth {
        let new_mono = || if on { Mono::Legato } else { Mono::Normal };
        let is_poly = match self.mode {
            Mode::Mono(ref mut mono, _) => {
                *mono = new_mono();
                false
            },
            Mode::Poly => true,
        };
        if is_poly {
            self.mode = Mode::Mono(new_mono(), empty_note_stack());
        }
        self
    }

    /// Add an oscillator to a Synth.
    #[inline]
    pub fn oscillator(mut self, oscillator: Oscillator) -> Synth {
        self.oscillators.push(oscillator);
        for voice in self.voices.iter_mut() {
            voice.oscillator_phases.push(0.0);
        }
        self
    }

    /// Add multiple oscillators to a Synth.
    #[inline]
    pub fn oscillators<I: Iterator<Item=Oscillator>>(mut self, oscillators: I) -> Synth {
        let len = self.oscillators.len();
        self.oscillators.extend(oscillators);
        let target_len = self.oscillators.len();
        for voice in self.voices.iter_mut() {
            voice.oscillator_phases.extend((len..target_len).map(|_| 0.0));
        }
        self
    }

    /// Set the Synth's duration.
    pub fn duration(mut self, duration: Duration) -> Synth {
        self.duration = duration;
        self
    }

    /// Set the amplitude for each channel.
    pub fn channels(mut self, channels: Vec<f32>) -> Synth {
        self.channels = channels;
        self
    }

    /// Set the amplitude of each channel according to a given stereo pan between -1.0 and 1.0.
    /// If the given value is outside the range -1.0..1.0, it will be clamped to range.
    /// The synth's number of channels will be set to two if it does not already have two.
    pub fn stereo_pan(mut self, pan: f32) -> Synth {
        let pan = if pan < -1.0 { -1.0 } else if pan > 1.0 { 1.0 } else { pan };
        let len = self.channels.len();
        if len > 2 {
            self.channels.truncate(2);
        } else if len < 2 {
            self.channels.extend((len..2).map(|_| 1.0));
        }
        let panned = stereo::pan(pan);
        self.channels[0] = panned[0];
        self.channels[1] = panned[1];
        self
    }

    /// Set the Synth's base pitch.
    pub fn base_pitch(mut self, base_pitch: BasePitch) -> Synth {
        self.base_pitch = base_pitch;
        self
    }

    /// Set the Synth's detune amount.
    pub fn detune(mut self, detune: f32) -> Synth {
        self.detune = detune;
        self
    }

    /// Set the Synth's spread amount.
    pub fn spread(mut self, spread: f32) -> Synth {
        self.spread = spread;
        self
    }

    /// Set the Synth's volume.
    pub fn volume(mut self, vol: f32) -> Synth {
        self.volume = vol;
        self
    }

    /// Set the Synth's portamento duration in milliseconds.
    pub fn portamento(mut self, portamento: f64) -> Synth {
        self.portamento = portamento;
        self
    }

    /// Set the loop data for the synth.
    pub fn loop_points(mut self, start: LoopStart, end: LoopEnd) -> Synth {
        self.loop_data = Some((start, end));
        self
    }

    /// Set the fade data for the synth.
    pub fn fade(mut self, attack: Attack, release: Release) -> Synth {
        self.fade_data = Some((attack, release));
        self
    }

    /// Set the start loop point.
    pub fn loop_start(mut self, start: LoopStart) -> Synth {
        let loop_data = match self.loop_data {
            Some((_, end)) => Some((start, end)),
            None => Some((start, 1.0))
        };
        self.loop_data = loop_data;
        self
    }

    /// Set the end loop point.
    pub fn loop_end(mut self, end: LoopEnd) -> Synth {
        let loop_data = match self.loop_data {
            Some((start, _)) => Some((start, end)),
            None => Some((0.0, end))
        };
        self.loop_data = loop_data;
        self
    }

    /// Set the attack.
    pub fn attack(mut self, attack: Attack) -> Synth {
        let fade_data = match self.fade_data {
            Some((_, release)) => Some((attack, release)),
            None => Some((attack, 0.0))
        };
        self.fade_data = fade_data;
        self
    }

    /// Set the release.
    pub fn release(mut self, release: Release) -> Synth {
        let fade_data = match self.fade_data {
            Some((attack, _)) => Some((attack, release)),
            None => Some((0.0, release))
        };
        self.fade_data = fade_data;
        self
    }

    /// Add an oscillator.
    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        for voice in self.voices.iter_mut() {
            voice.oscillator_phases.push(0.0);
        }
    }

    /// Remove and return the oscillator at the given idx.
    pub fn remove_oscillator(&mut self, idx: usize) -> Oscillator {
        for voice in self.voices.iter_mut() {
            voice.oscillator_phases.remove(idx);
        }
        self.oscillators.remove(idx)
    }

    /// Return whether or not there are any currently active voices.
    pub fn is_active(&self) -> bool {
        if self.is_paused { return false }
        self.voices.iter().any(|voice| voice.maybe_note.is_some())
    }

    /// Begin playback of a note. Synth will try to use a free `Voice` to do this.
    /// If no `Voice`s are free, the one playing the oldest note will be chosen to
    /// play the new note instead.
    #[inline]
    pub fn note_on(&mut self, note_hz: NoteHz, note_velocity: NoteVelocity) {
        let Synth { detune, portamento, ref mut mode, ref mut voices, .. } = *self;

        // Determine the starting frequency for the note.
        let start_freq = |maybe_last_hz: Option<pitch::calc::Hz>| {
            CurrentFreq::new(portamento, detune, note_hz, maybe_last_hz)
        };

        match *mode {

            Mode::Mono(mono, ref mut notes) => {

                // If some note is already playing, take it to use for portamento.
                let maybe_last_hz = match voices[0].maybe_note.as_ref() {
                    Some(&(NoteState::Playing, _, ref f, _)) => Some(f.hz()),
                    _ => None,
                };

                if let Some((NoteState::Playing, hz, _, _)) = voices[0].maybe_note.take() {
                    notes.push(hz);
                } else {
                    notes.clear();
                    for voice in voices.iter_mut() {
                        voice.reset_playheads();
                    }
                }
                if let Mono::Normal = mono {
                    for voice in voices.iter_mut() {
                        voice.reset_playheads();
                    }
                }
                for voice in voices.iter_mut() {
                    voice.note_on(note_hz, start_freq(maybe_last_hz), note_velocity);
                }
            },

            Mode::Poly => {

                // Construct the new CurrentFreq for the new note.
                let freq = {
                    // First, determine the current hz of the last note played if there is one.
                    let mut active = voices.iter().filter(|voice| voice.maybe_note.is_some());
                    fn hz_and_playhead(voice: &Voice) -> (pitch::calc::Hz, Playhead) {
                        let hz = voice.maybe_note.map(|(_, _, f, _)| f.hz()).unwrap();
                        (hz, voice.playhead)
                    }
                    let maybe_last_hz = active.next().map(|voice| {
                        active.fold(hz_and_playhead(voice), |(hz, playhead), voice| {
                            if voice.playhead > playhead { hz_and_playhead(voice) }
                            else { (hz, playhead) }
                        })
                    }).map(|(hz, _)| hz);
                    start_freq(maybe_last_hz)
                };

                // Find the right voice to play the note.
                let mut oldest: Option<&mut Voice> = None;
                let mut max_sample_count: i64 = 0;
                for voice in voices.iter_mut() {
                    if voice.maybe_note.is_none() {
                        voice.reset_playheads();
                        voice.note_on(note_hz, freq, note_velocity);
                        return;
                    }
                    else if voice.playhead >= max_sample_count {
                        max_sample_count = voice.playhead;
                        oldest = Some(voice);
                    }
                }
                if let Some(voice) = oldest {
                    voice.reset_playheads();
                    voice.note_on(note_hz, freq, note_velocity);
                }
            }

        }
    }

    /// Stop playback of the note that was triggered with the matching frequency.
    #[inline]
    pub fn note_off(&mut self, note_hz: NoteHz) {
        const HZ_VARIANCE: NoteHz = 0.0001;
        let (min_hz, max_hz) = (note_hz - HZ_VARIANCE, note_hz + HZ_VARIANCE);

        // Does the given hz match the note_off hz.
        let hz_match = |hz: NoteHz| hz > min_hz && hz < max_hz;

        // Does the voice's currently playing note match the note_off given.
        let is_match = |voice: &Voice| match voice.maybe_note {
            Some((NoteState::Playing, voice_note_hz, _, _)) => hz_match(voice_note_hz),
            _ => false,
        };

        let Synth { detune, portamento,  ref mut mode, ref mut voices, .. } = *self;

        match *mode {

            // If the synth is in a monophonic mode.
            Mode::Mono(mono, ref mut notes) => {
                if is_match(&mut voices[0]) {
                    if let Some((_, _, last_freq, vel)) = voices[0].maybe_note {
                        // If there's a note still on the stack, fall back to it.
                        if let Some(old_hz) = notes.pop() {

                            if let Mono::Normal = mono {
                                for voice in voices.iter_mut() {
                                    voice.reset_playheads();
                                }
                            }

                            // Determine the new CurrentFreq for the fallback note.
                            let hz = last_freq.hz();

                            // Play the popped stack note on all voices.
                            for voice in voices.iter_mut() {
                                let freq = CurrentFreq::new(portamento, detune, old_hz, Some(hz));
                                voice.note_on(old_hz, freq, vel);
                            }
                            return;
                        }
                    }
                    for voice in voices.iter_mut() {
                        voice.note_off();
                    }
                } else {
                    // If any notes in the note stack match the given note_off, remove them.
                    for i in (0..notes.len()).rev() {
                        if hz_match(notes[i]) {
                            notes.remove(i);
                        }
                    }
                }
            },

            // If the synth is in a polyphonic mode.
            Mode::Poly => {
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
        if let Mode::Mono(_, ref mut notes) = self.mode {
            notes.clear();
        }
        for voice in self.voices.iter_mut() {
            voice.stop();
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
            ref channels,
            ref mut voices,
            spread,
            duration,
            base_pitch,
            volume,
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

        // Determine the amplitude for each channel.
        let amp_per_channel = (0..settings.channels as usize).zip(channels.iter()).map(|(_, amp)| {
            *amp * volume
        }).collect::<Vec<_>>();

        // Prepare a Vec to use for calculating the pan for each voice.
        let mut voice_amp_per_channel = amp_per_channel.clone();

        // Is the given voice currently playing something.
        fn is_active(voice: &Voice) -> bool { voice.maybe_note.is_some() }

        // The number of voices to consider when calculating the pan spread.
        let num_voices = voices.iter().filter(|v| is_active(v)).count();

        // Request audio from each voice.
        for (i, voice) in voices.iter_mut().filter(|v| is_active(v)).enumerate() {

            // A working buffer which we will fill using the Voice.
            let mut working: Vec<S> = vec![Sample::zero(); settings.buffer_size()];

            // Fill the working buffer with the voice.
            voice.fill_buffer(&mut working,
                              settings,
                              oscillators,
                              duration,
                              base_pitch,
                              loop_data_samples.as_ref(),
                              fade_data_samples.as_ref());

            // If we have a stereo stream, calculate the spread.
            if settings.channels == 2 && spread > 0.0 {
                let pan = match num_voices {
                    1 => 0.0,
                    _ => ((i as f32 / (num_voices-1) as f32) - 0.5) * (spread * 2.0),
                };
                let panned = stereo::pan(pan);

                // Multiply the pan result with the amp_per_channel to get the voice's amp.
                voice_amp_per_channel[0] = amp_per_channel[0] * panned[0];
                voice_amp_per_channel[1] = amp_per_channel[1] * panned[1];
            }

            Sample::add_buffers(output, &working, &voice_amp_per_channel);
        }
    }

}

