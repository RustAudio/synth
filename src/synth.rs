//!
//!  synth.rs
//!
//!  Created by Mitchell Nordine at 03:37PM on July 02, 2014.
//!
//!
//!  Implementation of the `Synth` struct for basic multi-voice, multi-oscillator envelope synthesis.
//!

use dsp::{sample, Node as DspNode, Sample, Settings as DspSettings};
use mode::Mode;
use oscillator::{self, FreqWarp, Oscillator};
use note_freq::{NoteFreq, NoteFreqGenerator};
use panning::stereo;
use pitch;
use std::iter::repeat;
use time::{self, Ms};
use voice::{NoteVelocity, OscillatorState, Voice};


pub type Duration = time::calc::Ms;
pub type BasePitch = pitch::calc::Hz;
pub type LoopStart = f64;
pub type LoopEnd = f64;
pub type Attack = time::calc::Ms;
pub type Release = time::calc::Ms;
pub type Playhead = time::calc::Samples;

/// The `Synth` generates audio via a vector of `Voice`s, while a `Voice` generates audio via a
/// vector of `Oscillator`s, creating a small DSP tree.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Synth<M, NFG, W, A, F, FW> where NFG: NoteFreqGenerator {
    /// Oscillators for playback.
    pub oscillators: Vec<Oscillator<W, A, F, FW>>,
    /// The mode of note playback.
    pub mode: M,
    /// The amplitude for each channel.
    pub channels: Vec<f32>,
    /// The voices used by the Synth.
    /// - If the Synth is in Poly mode, it will play one voice at a time.
    /// - If the Synth is in Mono mode, it will play all voices at once.
    pub voices: Vec<Voice<NFG::NoteFreq>>,
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
    pub note_freq_gen: NFG,
    /// Data used for looping over a duration of the Synth.
    pub loop_data: Option<(LoopStart, LoopEnd)>,
    /// Data used for fading in / out from playback.
    pub fade_data: Option<(Attack, Release)>,
    /// Is the playback currently paused?
    pub is_paused: bool,
}


impl<M, NFG, W, A, F, FW> Synth<M, NFG, W, A, F, FW>
    where NFG: NoteFreqGenerator,
{

    /// Constructor for a new Synth.
    #[inline]
    pub fn new(mode: M, note_freq_gen: NFG) -> Self {
        const MS_300: Duration = 300.0;
        const C_1: BasePitch = 32.703;
        Synth {
            oscillators: Vec::new(),
            mode: mode,
            channels: Vec::from(&stereo::centre()[..]),
            voices: vec![Voice::new(0)],
            detune: 0.0,
            spread: 0.0,
            duration: MS_300,
            base_pitch: C_1,
            volume: 1.0,
            note_freq_gen: note_freq_gen,
            loop_data: None,
            fade_data: None,
            is_paused: false,
        }
    }

    /// Return the synth with the given number of voices.
    #[inline]
    pub fn num_voices(mut self, num_voices: usize) -> Self {
        self.set_num_voices(num_voices);
        self
    }

    /// Set the number of voices that the Synth shall use.
    #[inline]
    pub fn set_num_voices(&mut self, num_voices: usize) {
        if num_voices == 0 {
            println!("A Synth must have at least one voice, but the requested number is 0.");
        } else {
            let len = self.voices.len();
            if len < num_voices {
                let last_voice = self.voices[len-1].clone();
                self.voices.extend(repeat(last_voice).take(num_voices - len));
            } else if len > num_voices {
                self.voices.truncate(num_voices);
            }
        }
    }

    /// Changes the mode of the Synth.
    #[inline]
    pub fn mode<NewM>(self, new_mode: NewM) -> Synth<NewM, NFG, W, A, F, FW> {
        let Synth {
            oscillators,
            channels,
            voices,
            detune,
            spread,
            duration,
            base_pitch,
            volume,
            note_freq_gen,
            loop_data,
            fade_data,
            is_paused,
            ..
        } = self;
        Synth {
            oscillators: oscillators,
            mode: new_mode,
            channels: channels,
            voices: voices,
            detune: detune,
            spread: spread,
            duration: duration,
            base_pitch: base_pitch,
            volume: volume,
            note_freq_gen: note_freq_gen,
            loop_data: loop_data,
            fade_data: fade_data,
            is_paused: is_paused,
        }
    }

    /// Add an oscillator to a Synth.
    #[inline]
    pub fn oscillator(mut self, oscillator: Oscillator<W, A, F, FW>) -> Self {
        self.add_oscillator(oscillator);
        self
    }

    /// Add multiple oscillators to a Synth.
    #[inline]
    pub fn oscillators<I: Iterator<Item=Oscillator<W, A, F, FW>>>(mut self, oscillators: I) -> Self
    {
        let len = self.oscillators.len();
        self.oscillators.extend(oscillators);
        let target_len = self.oscillators.len();
        for voice in self.voices.iter_mut() {
            voice.oscillator_states.extend((len..target_len).map(|_| OscillatorState::new()));
        }
        self
    }

    /// Set the Synth's duration.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the amplitude for each channel.
    pub fn channels(mut self, channels: Vec<f32>) -> Self {
        self.channels = channels;
        self
    }

    /// Set the amplitude of each channel according to a given stereo pan between -1.0 and 1.0.
    /// If the given value is outside the range -1.0..1.0, it will be clamped to range.
    /// The synth's number of channels will be set to two if it does not already have two.
    pub fn stereo_pan(mut self, pan: f32) -> Self {
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
    pub fn base_pitch(mut self, base_pitch: BasePitch) -> Self {
        self.base_pitch = base_pitch;
        self
    }

    /// Set the Synth's detune amount.
    pub fn detune(mut self, detune: f32) -> Self {
        self.detune = detune;
        self
    }

    /// Set the Synth's spread amount.
    pub fn spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// Set the Synth's volume.
    pub fn volume(mut self, vol: f32) -> Self {
        self.volume = vol;
        self
    }

    /// Set the Synth's portamento duration in milliseconds.
    pub fn note_freq_generator<NewNFG>(self, generator: NewNFG) -> Synth<M, NewNFG, W, A, F, FW>
        where NewNFG: NoteFreqGenerator,
    {
        let Synth {
            oscillators,
            mode,
            channels,
            voices,
            detune,
            spread,
            duration,
            base_pitch,
            volume,
            loop_data,
            fade_data,
            is_paused,
            ..
        } = self;

        let voices = voices.into_iter().map(|voice| {
            let Voice { oscillator_states, maybe_note, playhead, loop_playhead } = voice;
            let maybe_note = maybe_note.map(|(state, hz, _, vel)| {
                let note_freq = generator.generate(hz, detune, None);
                (state, hz, note_freq, vel)
            });
            Voice {
                oscillator_states: oscillator_states,
                maybe_note: maybe_note,
                playhead: playhead,
                loop_playhead: loop_playhead,
            }
        }).collect();

        Synth {
            oscillators: oscillators,
            mode: mode,
            channels: channels,
            voices: voices,
            detune: detune,
            spread: spread,
            duration: duration,
            base_pitch: base_pitch,
            volume: volume,
            note_freq_gen: generator,
            loop_data: loop_data,
            fade_data: fade_data,
            is_paused: is_paused,
        }
    }

    /// Set the loop data for the synth.
    pub fn loop_points(mut self, start: LoopStart, end: LoopEnd) -> Self {
        self.loop_data = Some((start, end));
        self
    }

    /// Set the fade data for the synth.
    pub fn fade(mut self, attack: Attack, release: Release) -> Self {
        self.fade_data = Some((attack, release));
        self
    }

    /// Set the start loop point.
    pub fn loop_start(mut self, start: LoopStart) -> Self {
        let loop_data = match self.loop_data {
            Some((_, end)) => Some((start, end)),
            None => Some((start, 1.0))
        };
        self.loop_data = loop_data;
        self
    }

    /// Set the end loop point.
    pub fn loop_end(mut self, end: LoopEnd) -> Self {
        let loop_data = match self.loop_data {
            Some((start, _)) => Some((start, end)),
            None => Some((0.0, end))
        };
        self.loop_data = loop_data;
        self
    }

    /// Set the attack.
    pub fn attack(mut self, attack: Attack) -> Self {
        let fade_data = match self.fade_data {
            Some((_, release)) => Some((attack, release)),
            None => Some((attack, 0.0))
        };
        self.fade_data = fade_data;
        self
    }

    /// Set the release.
    pub fn release(mut self, release: Release) -> Self {
        let fade_data = match self.fade_data {
            Some((attack, _)) => Some((attack, release)),
            None => Some((0.0, release))
        };
        self.fade_data = fade_data;
        self
    }

    /// Add an oscillator.
    pub fn add_oscillator(&mut self, oscillator: Oscillator<W, A, F, FW>) {
        self.oscillators.push(oscillator);
        for voice in self.voices.iter_mut() {
            voice.oscillator_states.push(OscillatorState::new());
        }
    }

    /// Remove and return the oscillator at the given idx.
    pub fn remove_oscillator(&mut self, idx: usize) -> Oscillator<W, A, F, FW> {
        for voice in self.voices.iter_mut() {
            voice.oscillator_states.remove(idx);
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
    pub fn note_on<T>(&mut self, note_hz: T, note_vel: NoteVelocity)
        where M: Mode,
              T: Into<pitch::Hz>
    {
        let Synth { detune, ref note_freq_gen, ref mut mode, ref mut voices, .. } = *self;
        mode.note_on(note_hz.into().hz(), note_vel, detune, note_freq_gen, voices);
    }

    /// Stop playback of the note that was triggered with the matching frequency.
    #[inline]
    pub fn note_off<T>(&mut self, note_hz: T)
        where M: Mode,
              T: Into<pitch::Hz>
    {
        let Synth { detune, ref note_freq_gen,  ref mut mode, ref mut voices, .. } = *self;
        mode.note_off(note_hz.into().hz(), detune, note_freq_gen, voices);
    }

    /// Pause playback.
    #[inline]
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Unpause playback.
    #[inline]
    pub fn unpause(&mut self) {
        self.is_paused = false;
    }

    /// Stop playback and clear the current notes.
    #[inline]
    pub fn stop(&mut self)
        where M: Mode,
    {
        self.mode.stop();
        for voice in self.voices.iter_mut() {
            voice.stop();
        }
    }

}

impl<S, M, NFG, W, A, F, FW> DspNode<S> for Synth<M, NFG, W, A, F, FW> where
    S: Sample + sample::Duplex<f32>,
    NFG: NoteFreqGenerator,
    W: oscillator::Waveform,
    A: oscillator::Amplitude,
    F: oscillator::Frequency,
    FW: FreqWarp,
{

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
        fn is_active<NF>(voice: &Voice<NF>) -> bool { voice.maybe_note.is_some() }

        // The number of voices to consider when calculating the pan spread.
        let num_voices = voices.iter().filter(|v| is_active(v)).count();

        // Request audio from each voice.
        for (i, voice) in voices.iter_mut().filter(|v| is_active(v)).enumerate() {

            // A working buffer which we will fill using the Voice.
            let mut working: Vec<S> = vec![S::equilibrium(); settings.buffer_size()];

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

            sample::buffer::add_with_amp_per_channel(output, &working, &voice_amp_per_channel);
        }
    }

}

