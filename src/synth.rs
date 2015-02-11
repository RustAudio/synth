//!
//!  synth.rs
//!
//!  Created by Mitchell Nordine at 03:37PM on July 02, 2014.
//!
//!

//! Implementation of the `Synth` struct for basic multi-voice, multi-oscillator envelope synthesis.

use dsp::Node as DspNode;
use dsp::Settings as DspSettings;
use dsp::{AudioBuffer, DspBuffer};
use oscillator::Oscillator;
use pitch;
use time::{self, Ms};
use voice::{Voice, NoteDuration};

pub type Duration = time::calc::Ms;
pub type BasePitch = pitch::calc::Hz;
pub type LoopStart = f64;
pub type LoopEnd = f64;
pub type Attack = time::calc::Ms;
pub type Release = time::calc::Ms;
pub type Playhead = time::calc::Samples;
pub type NoteHz = pitch::calc::Hz;

/// The `Synth` generates audio via a vector of `Voice`s,
/// while a `Voice` generates audio via a vector of
/// `Oscillator`s, creating a small DSP tree.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Synth {
    /// Vector of inputs to the Synth. Polyphonic synth instruments
    /// will utilise a vector with more than one voice.
    pub voices: Vec<Voice>,
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
}

const MS_300: Duration = 300.0;
const C_1: BasePitch = 32.703;

impl Synth {

    /// Constructor for Synth instrument.
    pub fn new(oscillators: Vec<Oscillator>,
               num_voices: usize,
               base_pitch: BasePitch,
               duration: Duration,
               vol: f32,
               normaliser: f32,
               loop_data: Option<(LoopStart, LoopEnd)>,
               fade_data: Option<(Attack, Release)>) -> Synth {
        let voices = (0..num_voices).map(|_| Voice::new(oscillators.clone())).collect();
        Synth::from_voices(voices, base_pitch, duration, vol, normaliser, loop_data, fade_data)
    }

    /// Construct a Synth from it's Voices rather than Oscillators and a number of voices.
    pub fn from_voices(voices: Vec<Voice>,
                       base_pitch: BasePitch,
                       duration: Duration,
                       vol: f32,
                       normaliser: f32,
                       loop_data: Option<(LoopStart, LoopEnd)>,
                       fade_data: Option<(Attack, Release)>) -> Synth {
        Synth {
            voices: voices,
            duration: duration,
            base_pitch: base_pitch,
            vol: vol,
            normaliser: normaliser,
            loop_data: loop_data,
            fade_data: fade_data,
        }
    }

    /// Default constructor for a Synth instrument.
    pub fn default() -> Synth {
        let voices = vec![ Voice::default() ];
        Synth::from_voices(voices, C_1, MS_300, 1.0, 1.0, None, None)
    }

    /// Constructor for quick and easy testing of Synth Instrument.
    pub fn test_demo() -> Synth {
        let voices = vec![ Voice::test_demo() ];
        Synth::from_voices(voices, C_1, MS_300, 1.0, 1.0, None, None)
    }

    /// Add a default oscillator.
    pub fn add_oscillator(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.oscillators.push(Oscillator::new())
        }
    }

    /// Remove an oscillator.
    pub fn remove_oscillator(&mut self, idx: usize) {
        assert!(self.voices[0].oscillators.len() > idx,
                "Synth::remove_oscillator - the given idx ({}) is greater than \
                the number of oscillators in the first voice!", idx);
        for voice in self.voices.iter_mut() {
            voice.oscillators.remove(idx);
        }
        if self.voices[0].oscillators.len() == 0 {
            self.add_oscillator()
        }
    }

    /// Return whether or not there are any currently active voices.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|voice| voice.maybe_note.is_some())
    }

    /// Trigger playback with an optional given note.
    #[inline]
    pub fn play_note(&mut self, maybe_note: (NoteDuration, NoteHz)) {
        let (duration, hz) = maybe_note;
        let note_freq_multi = hz as f64 / self.base_pitch as f64;
        let mut oldest: Option<&mut Voice> = None;
        let mut max_sample_count: i64 = 0;
        for voice in self.voices.iter_mut() {
            if voice.maybe_note.is_none() {
                voice.play_note((duration, note_freq_multi));
                return;
            }
            else if voice.playhead >= max_sample_count {
                max_sample_count = voice.playhead;
                oldest = Some(voice);
            }
        }
        if let Some(voice) = oldest {
            voice.play_note((duration, note_freq_multi))
        }
    }

}

impl<B> DspNode<B> for Synth where B: DspBuffer {

    #[inline]
    fn audio_requested(&mut self, output: &mut B, settings: DspSettings) {
        if !self.is_active() { return }
        let (frames, channels) = (settings.frames as usize, settings.channels as usize);
        let buffer_size = frames * channels;
        let sample_hz = settings.sample_hz as f64;
        let Synth {
            ref mut voices,
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
            use std::num::Float;
            ((start_perc * duration as f64).round() as time::calc::Samples,
             (end_perc * duration as f64).round() as time::calc::Samples)
        });

        // Convert the fade durations from milliseconds to samples.
        let fade_data_samples = fade_data.map(|(attack_ms, release_ms)| {
            (Ms(attack_ms).samples(sample_hz), Ms(release_ms).samples(sample_hz))
        });

        let amp_multi = vol * normaliser;
        let vol_per_channel = [amp_multi; 2];

        // Request audio from each voice and sum them together.
        for voice in voices.iter_mut() {
            let mut working: B = AudioBuffer::zeroed(buffer_size);
            voice.fill_buffer(&mut working,
                              settings,
                              duration,
                              loop_data_samples.as_ref(),
                              fade_data_samples.as_ref());
            for i in 0..frames {
                for j in 0..channels {
                    use dsp::Sample;
                    let idx = i * channels + j;
                    let working_sample = working.val(idx).mul_amp(vol_per_channel[j]);
                    *output.get_mut(idx) = *output.get_mut(idx) + working_sample;
                }
            }
        }
    }

}

    // fn audio_requested(&mut self, output: &mut B, settings: DspSettings) {
    //     let (frames, channels) = (settings.frames as usize, settings.channels as usize);
    //     let Synth {
    //         ref mut voices,
    //         loop_data: (loop_start, loop_end, ref mut loop_playhead),
    //         attack,
    //         release,
    //         ref mut playhead,
    //         ref mut is_playing,
    //         duration,
    //         note_duration,
    //         note_freq_multi,
    //         normaliser,
    //         vol,
    //     } = *self;
    //     let duration = Ms(duration).samples(settings.sample_hz);

    //     let mut value: f32;

    //     for i in range(0, frames) {

    //         if *is_playing && loop_playhead < duration {
    //             // Sum the amplitude of each oscillator at the given ratio.
    //             let ratio = loop_playhead as f64 / duration as f64;
    //             value = voices.iter_mut().fold(0.0, |total, voice| {
    //                 let note = voice.note;
    //                 if let Some(note_duration, note_hz) = note {
    //                     total + voice.oscillators.iter_mut().fold(0.0, |total, oscillator| {
    //                         total + oscillator.amp_at_ratio(ratio, note_freq_multi, settings.sample_hz)
    //                     })
    //                 } else {
    //                     total
    //                 }
    //             }) * normaliser * vol;
    //         } else {
    //             // If not playing, just assign the value as 0.
    //             value = 0.0;
    //         }

    //         // Assign the value to each channel.
    //         for j in range(0, channels) {
    //             output[i * channels + j] = value;
    //         }

    //         // Iterate the loop_playhead. If the loop_playhead passes the loop_end,
    //         // reset the playhead to the start.
    //         *loop_playhead += 1;
    //         if *loop_playhead >= loop_end {
    //             *loop_playhead = (*loop_playhead - loop_end) + loop_start;
    //         }

    //         // Iterate the playhead. If the playhead passes the duration of the instrument or
    //         // the note that is currently being played, reset the playhead and stop playback.
    //         *playhead += 1;
    //         if *playhead >= note_duration + release ||
    //         *loop_playhead > duration {
    //             *is_playing = false;
    //             *playhead = 0;
    //         }

    //     }

    // }

// impl_pitch!(Synth, inst_data.base_pitch)

// impl Instrument for Synth {
// 
//     /// Get a reference to the Data struct.
//     fn get_instrument_data<'a>(&'a self) -> &'a Data { &self.inst_data }
// 
//     /// Get a mutable reference to the Data struct.
//     fn get_instrument_data_mut<'a>(&'a mut self) -> &'a mut Data { &mut self.inst_data }
// 
//     /// Set normaliser.
//     fn set_normaliser(&mut self, normaliser: f32) {
//         for voice in self.voices.iter_mut() {
//             for oscillator in voice.oscillators.iter_mut() {
//                 oscillator.normaliser = normaliser;
//             }
//         }
//         self.inst_data.analysis.normaliser = normaliser;
//     }
// 
//     /// Set the automixer for the instrument.
//     fn set_auto_mixer(&mut self, auto_mixer: f32) {
//         for voice in self.voices.iter_mut() {
//             for oscillator in voice.oscillators.iter_mut() {
//                 oscillator.auto_mixer = auto_mixer;
//             }
//         }
//         self.inst_data.auto_mixer = auto_mixer;
//     }
// 
// }
// 
// 
// impl Fadable for Synth {
//     impl_fadable_get_data!(inst_data.fade_data)
//     /// Return a vector of mutable references to all `Fadable` children.
//     fn get_fadable_children<'a>(&'a mut self) -> Vec<&'a mut Fadable> {
//         let num_children = self.voices.iter().fold(0u, |a, ref b| a + b.oscillators.len());
//         self.voices.iter_mut().fold(Vec::with_capacity(num_children), |mut vec: Vec<&'a mut Fadable + 'a>, voice| {
//             let temp: Vec<&mut Fadable> = voice.oscillators.iter_mut().map(|osc| osc as &mut Fadable).collect();
//             vec.extend(temp.into_iter()); vec
//         })
//     }
// }
// 
// impl Loopable for Synth {
//     impl_loopable_get_data!(inst_data.loop_data)
//     /// Return a vector of mutable references to all `Loopable` children.
//     fn get_loopable_children<'a>(&'a mut self) -> Vec<&'a mut Loopable> {
//         let num_children = self.voices.iter().fold(0u, |a, ref b| a + b.oscillators.len());
//         self.voices.iter_mut().fold(Vec::with_capacity(num_children), |mut vec: Vec<&'a mut Loopable + 'a>, voice| {
//             let temp: Vec<&mut Loopable> = voice.oscillators.iter_mut().map(|osc| osc as &mut Loopable).collect();
//             vec.extend(temp.into_iter()); vec
//         })
//     }
// }
// 
// impl Playable for Synth {
// 
//     impl_playable_get_children!(voices)
// 
//     /// Start the playback of the Instrument type.
//     fn play(&mut self, note: Option<&Note>) {
//         let mut oldest: Option<&mut Voice> = None;
//         let mut max_sample_count: i64 = 0;
//         let hz = self.hz();
//         self.set_note_freq_multi(note.map_or(1.0, |n| n.hz() / hz));
//         for voice in self.voices.iter_mut() {
//             if !voice.is_playing() {
//                 voice.play(note);
//                 return;
//             }
//             if voice.oscillators[0].playhead >= max_sample_count {
//                 max_sample_count = voice.oscillators[0].playhead;
//                 oldest = Some(voice);
//             }
//         }
//         match oldest { Some(voice) => voice.play(note), None => () }
//     }
// 
//     /// Return whether or not the Synth is currently playing.
//     fn is_playing(&self) -> bool {
//         for voice in self.voices.iter() {
//             if voice.is_playing() {
//                 return true;
//             }
//         }
//         false
//     }
// 
// }
// 

