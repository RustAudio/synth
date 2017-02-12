//!  Implementation of the `Synth` struct for basic multi-voice, multi-oscillator envelope
//!  synthesis.

use instrument::{self, Instrument, NoteFreq, NoteFreqGenerator};
use instrument::unit::NoteVelocity;
use oscillator::{self, Amplitude, Frequency, FreqWarp, Oscillator, Waveform};
use panning::stereo;
use pitch;
use sample::{self, Frame, Sample};
use std;
use time;


pub type LoopStartPerc = f64;
pub type LoopEndPerc = f64;
pub type Duration = time::Ms;
pub type BasePitch = pitch::calc::Hz;


/// The `Synth` generates audio via a vector of `Voice`s, while a `Voice` generates audio via a
/// vector of `Oscillator`s, creating a small DSP tree.
#[derive(Clone, Debug)]
pub struct Synth<M, NFG, W, A, F, FW>
    where NFG: NoteFreqGenerator,
{
    /// Oscillators for playback.
    pub oscillators: Vec<Oscillator<W, A, F, FW>>,
    /// Per-`instrument::Voice` state that is unique to the `Synth`.
    pub voices: Vec<Voice>,
    /// The instrument used for performing the synth.
    pub instrument: Instrument<M, NFG>,
    /// An amplitude multiplier.
    pub volume: f32,
    /// The amount each voice should be spread across the available channels.
    pub spread: f32,
    /// The start and end points that will be looped.
    pub loop_points: Option<(LoopStartPerc, LoopEndPerc)>,
    /// Duration of the Synth instrument in samples.
    pub duration_ms: Duration,
    /// Base pitch of the Synth instrument in Steps.
    pub base_pitch: BasePitch,
}

impl<M, NFG, W, A, F, FW> PartialEq for Synth<M, NFG, W, A, F, FW>
    where M: PartialEq,
          NFG: PartialEq + NoteFreqGenerator,
          W: PartialEq,
          A: PartialEq,
          F: PartialEq,
          FW: PartialEq,
          Instrument<M, NFG>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.oscillators == other.oscillators
        && self.voices == other.voices
        && self.instrument == other.instrument
        && self.volume == other.volume
        && self.spread == other.spread
        && self.loop_points == other.loop_points
        && self.duration_ms == other.duration_ms
        && self.base_pitch == other.base_pitch
    }
}

/// Per-`instrument::Voice` state that is unique to the `Synth`.
#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub loop_playhead: time::calc::Samples,
    /// The state of each oscillator unique to each voice.
    pub oscillator_states: oscillator::StatePerVoice,
}

/// An iterator that uniquely borrows the `Synth` and endlessly yields `Frame`s.
///
/// Each frame, parts of the `Synth`'s internal state are stepped forward accordingly, including:
///
/// - Oscillator `FreqWarp` phase.
/// - Oscillator `Waveform` phase.
/// - Loop playhead per-voice.
/// - Instrument note interpolation (`Portamento`, `Attack` and `Release` playheads).
pub struct Frames<'a, FRM, NF: 'a, W: 'a, A: 'a, F: 'a, FW: 'a> {
    sample_hz: time::SampleHz,
    oscillators: &'a mut [Oscillator<W, A, F, FW>],
    voices: &'a mut [Voice],
    loop_points: Option<(time::calc::Samples, time::calc::Samples)>,
    instrument_frames: instrument::Frames<'a, NF>,
    duration: time::calc::Samples,
    base_pitch: BasePitch,
    volume: f32,
    spread: f32,
    frame: std::marker::PhantomData<FRM>,
}


impl<NFG, W, A, F, FW> Synth<instrument::mode::Mono, NFG, W, A, F, FW>
    where NFG: NoteFreqGenerator,
{
    pub fn retrigger(nfg: NFG) -> Self {
        Self::new(instrument::mode::Mono::retrigger(), nfg)
    }
}

impl<NFG, W, A, F, FW> Synth<instrument::mode::Mono, NFG, W, A, F, FW>
    where NFG: NoteFreqGenerator,
{
    pub fn legato(nfg: NFG) -> Self {
        Self::new(instrument::mode::Mono::legato(), nfg)
    }
}

impl<NFG, W, A, F, FW> Synth<instrument::mode::Poly, NFG, W, A, F, FW>
    where NFG: NoteFreqGenerator,
{
    pub fn poly(nfg: NFG) -> Self {
        Self::new(instrument::mode::Poly, nfg)
    }
}

impl<M, NFG, W, A, F, FW> Synth<M, NFG, W, A, F, FW>
    where NFG: NoteFreqGenerator,
{

    /// Constructor for a new Synth.
    #[inline]
    pub fn new(mode: M, note_freq_gen: NFG) -> Self {
        const MS_300: Duration = time::Ms(300.0);
        const C_1: BasePitch = 32.703;
        let instrument = Instrument::new(mode, note_freq_gen);
        let n_voices = instrument.voices.len();
        let default_voice = Voice {
            loop_playhead: 0,
            oscillator_states: oscillator::StatePerVoice(Vec::new()),
        };
        Synth {
            oscillators: Vec::new(),
            voices: vec![default_voice; n_voices],
            //channels: Vec::from(&stereo::centre()[..]),
            volume: 1.0,
            spread: 0.0,
            duration_ms: MS_300,
            base_pitch: C_1,
            loop_points: None,
            instrument: instrument,
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
        self.instrument.set_num_voices(num_voices);
        if num_voices == 0 {
            println!("A Synth must have at least one voice, but the requested number is 0.");
        } else {
            let len = self.voices.len();
            if len < num_voices {
                let last_voice = self.voices[len-1].clone();
                let extension = std::iter::repeat(last_voice).take(num_voices - len);
                self.voices.extend(extension);
            } else if len > num_voices {
                self.voices.truncate(num_voices);
            }
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
        for &mut Voice { ref mut oscillator_states, .. } in &mut self.voices {
            let new_states = (len..target_len).map(|_| oscillator::State::new());
            oscillator_states.0.extend(new_states);
        }
        self
    }

    /// Set the Synth's duration.
    pub fn duration<D>(mut self, duration_ms: D) -> Self
        where D: Into<time::Ms>,
    {
        self.duration_ms = duration_ms.into();
        self
    }

    // /// Set the amplitude for each channel.
    // pub fn channels(mut self, channels: Vec<f32>) -> Self {
    //     self.channels = channels;
    //     self
    // }

    // /// Set the amplitude of each channel according to a given stereo pan between -1.0 and 1.0.
    // /// If the given value is outside the range -1.0..1.0, it will be clamped to range.
    // /// The synth's number of channels will be set to two if it does not already have two.
    // pub fn stereo_pan(mut self, pan: f32) -> Self {
    //     let pan = if pan < -1.0 { -1.0 } else if pan > 1.0 { 1.0 } else { pan };
    //     let len = self.channels.len();
    //     if len > 2 {
    //         self.channels.truncate(2);
    //     } else if len < 2 {
    //         self.channels.extend((len..2).map(|_| 1.0));
    //     }
    //     let panned = stereo::pan(pan);
    //     self.channels[0] = panned[0];
    //     self.channels[1] = panned[1];
    //     self
    // }

    /// Set the Synth's base pitch.
    pub fn base_pitch(mut self, base_pitch: BasePitch) -> Self {
        self.base_pitch = base_pitch;
        self
    }

    /// Set the Synth's detune amount.
    pub fn detune(mut self, detune: f32) -> Self {
        self.instrument.detune = detune;
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

    /// Convert `Self` into a new `Synth` with the given NoteFreqGenerator.
    pub fn note_freq_generator(self, generator: NFG) -> Self {
        self.map_instrument(|inst| inst.note_freq_generator(generator))
    }

    /// Set the loop data for the synth.
    pub fn loop_points(mut self, start: LoopStartPerc, end: LoopEndPerc) -> Self {
        self.loop_points = Some((start, end));
        self
    }

    /// Set the fade data for the synth.
    pub fn fade<Attack, Release>(self, attack: Attack, release: Release) -> Self
        where Attack: Into<time::Ms>,
              Release: Into<time::Ms>,
    {
        self.map_instrument(|inst| inst.fade(attack, release))
    }

    /// Set the start loop point.
    pub fn loop_start(mut self, start: LoopStartPerc) -> Self {
        self.loop_points = self.loop_points.map(|(_, end)| (start, end)).or(Some((start, 1.0)));
        self
    }

    /// Set the end loop point.
    pub fn loop_end(mut self, end: LoopEndPerc) -> Self {
        self.loop_points = self.loop_points.map(|(start, _)| (start, end)).or(Some((1.0, end)));
        self
    }

    /// Set the attack in milliseconds.
    pub fn attack<Attack>(self, attack: Attack) -> Self
        where Attack: Into<time::Ms>,
    {
        self.map_instrument(|inst| inst.attack(attack))
    }

    /// Set the release in milliseconds.
    pub fn release<Release>(self, release: Release) -> Self
        where Release: Into<time::Ms>,
    {
        self.map_instrument(|inst| inst.release(release))
    }

    /// Add an oscillator.
    pub fn add_oscillator(&mut self, oscillator: Oscillator<W, A, F, FW>) {
        self.oscillators.push(oscillator);
        for voice in &mut self.voices {
            voice.oscillator_states.0.push(oscillator::State::new());
        }
    }

    /// Remove and return the oscillator at the given idx.
    pub fn remove_oscillator(&mut self, idx: usize) -> Oscillator<W, A, F, FW> {
        for voice in &mut self.voices {
            voice.oscillator_states.0.remove(idx);
        }
        self.oscillators.remove(idx)
    }

    /// Return whether or not there are any currently active voices.
    pub fn is_active(&self) -> bool {
        self.instrument.is_active()
    }

    /// Begin playback of a note. Synth will try to use a free `Voice` to do this.
    /// If no `Voice`s are free, the one playing the oldest note will be chosen to
    /// play the new note instead.
    #[inline]
    pub fn note_on<T>(&mut self, note_hz: T, note_vel: NoteVelocity)
        where M: instrument::Mode,
              T: Into<pitch::Hz>
    {
        self.instrument.note_on(note_hz.into().hz(), note_vel);
    }

    /// Stop playback of the note that was triggered with the matching frequency.
    #[inline]
    pub fn note_off<T>(&mut self, note_hz: T)
        where M: instrument::Mode,
              T: Into<pitch::Hz>
    {
        self.instrument.note_off(note_hz.into().hz());
    }

    /// Stop playback and clear the current notes.
    #[inline]
    pub fn stop(&mut self)
        where M: instrument::Mode,
    {
        self.instrument.stop();
        for voice in &mut self.voices {
            for osc_state in &mut voice.oscillator_states.0 {
                *osc_state = oscillator::State::new();
            }
        }
    }

    /// Map the `Instrument` to a new `Instrument` in place.
    ///
    /// This is useful for providing wrapper builder methods for the Synth.
    #[inline]
    pub fn map_instrument<Map, NewM, NewNFG>(self, map: Map) -> Synth<NewM, NewNFG, W, A, F, FW>
        where Map: FnOnce(Instrument<M, NFG>) -> Instrument<NewM, NewNFG>,
              NewNFG: NoteFreqGenerator,
    {
        let Synth {
            oscillators,
            voices,
            duration_ms,
            base_pitch,
            volume,
            spread,
            instrument,
            loop_points,
        } = self;

        Synth {
            oscillators: oscillators,
            voices: voices,
            volume: volume,
            spread: spread,
            duration_ms: duration_ms,
            base_pitch: base_pitch,
            loop_points: loop_points,
            instrument: map(instrument)
        }
    }

    /// Produces an `Iterator` that endlessly yields new `Frame`s
    pub fn frames<FRM>(&mut self, sample_hz: f64) -> Frames<FRM, NFG::NoteFreq, W, A, F, FW>
        where FRM: Frame,
              <FRM::Sample as Sample>::Float: sample::FromSample<f32>,
              <FRM::Sample as Sample>::Signed: sample::FromSample<f32>,
    {
        let Synth {
            ref mut oscillators,
            ref mut voices,
            ref mut instrument,
            duration_ms,
            base_pitch,
            loop_points,
            spread,
            volume,
        } = *self;

        // Convert the duration from milliseconds to samples.
        let duration = duration_ms.samples(sample_hz);

        // Convert the loop points from duration percentages to samples.
        let loop_points_samples = loop_points.map(|(start_perc, end_perc)| {
            ((start_perc * duration as f64).round() as time::calc::Samples,
             (end_perc * duration as f64).round() as time::calc::Samples)
        });

        Frames {
            sample_hz: sample_hz,
            oscillators: oscillators,
            voices: voices,
            duration: duration,
            base_pitch: base_pitch,
            loop_points: loop_points_samples,
            instrument_frames: instrument.frames(sample_hz),
            spread: spread,
            volume: volume,
            frame: std::marker::PhantomData,
        }
    }

    /// Additively fill the given slice of `Frame`s with the `Synth::frames` method.
    pub fn fill_slice<FRM>(&mut self, output: &mut [FRM], sample_hz: f64)
        where FRM: sample::Frame,
              <FRM::Sample as Sample>::Float: sample::FromSample<f32>,
              <FRM::Sample as Sample>::Signed: sample::FromSample<f32>,
              M: instrument::Mode,
              NFG: instrument::NoteFreqGenerator,
              W: oscillator::Waveform,
              A: oscillator::Amplitude,
              F: oscillator::Frequency,
              FW: oscillator::FreqWarp,
    {
        let mut frames = self.frames::<FRM>(sample_hz);
        sample::slice::map_in_place(output, |f| {
            f.zip_map(frames.next_frame(), |a, b| a.add_amp(b.to_sample()))
        });
    }

}


impl<'a, FRM, NF, W, A, F, FW> Frames<'a, FRM, NF, W, A, F, FW>
    where FRM: Frame,
          <FRM::Sample as Sample>::Float: sample::FromSample<f32>,
          <FRM::Sample as Sample>::Signed: sample::FromSample<f32>,
          NF: NoteFreq,
          W: Waveform,
          A: Amplitude,
          F: Frequency,
          FW: FreqWarp,
{
    /// Yields the next frame
    #[inline]
    pub fn next_frame(&mut self) -> FRM {
        let Frames {
            ref mut oscillators,
            ref mut instrument_frames,
            ref mut voices,
            sample_hz,
            loop_points,
            duration,
            base_pitch,
            volume,
            spread,
            ..
        } = *self;

        // Count the number of voices currently playing a note.
        let num_active_voices = instrument_frames.num_active_voices();
        let frame_per_voice = instrument_frames.next_frame_per_voice();
        let iter = voices.iter_mut()
            .zip(frame_per_voice)
            .filter_map(|(v, amp_hz)| amp_hz.map(|amp_hz| (v, amp_hz)))
            .enumerate();
        let should_spread = FRM::n_channels() == 2 && spread > 0.0;

        let mut frame = FRM::equilibrium();
        for (i, (voice, (amp, hz))) in iter {
            let Voice { ref mut loop_playhead, ref mut oscillator_states } = *voice;
            if *loop_playhead < duration {
                let freq_multi = hz as f64 / base_pitch as f64;
                let playhead_perc = *loop_playhead as f64 / duration as f64;

                let osc_iter = oscillators.iter_mut().zip(oscillator_states.0.iter_mut());
                let wave = osc_iter.fold(0.0, |amp, (osc, state)| {
                    amp + osc.next_frame_amp(sample_hz, playhead_perc, freq_multi, state)
                }) * amp;

                // If we have a stereo stream, calculate the spread.
                frame = if should_spread {
                    let pan = match num_active_voices {
                        1 => 0.0,
                        _ => ((i as f32 / (num_active_voices-1) as f32) - 0.5) * (spread * 2.0),
                    };
                    let panned = stereo::pan(pan);

                    // Multiply the pan result with the amp_per_channel to get the voice's amp.
                    FRM::from_fn(|idx| {
                        let amp = wave * panned[idx];
                        frame.channel(idx).unwrap().add_amp(amp.to_sample())
                    })
                } else {
                    frame.map(|s| s.add_amp(wave.to_sample()))
                };

                // Iterate the loop_playhead. If the loop_playhead passes the loop_end, reset the
                // playhead to the start.
                *loop_playhead += 1;
                if let Some((loop_start, loop_end)) = loop_points {
                    if *loop_playhead >= loop_end {
                        *loop_playhead = (*loop_playhead - loop_end) + loop_start;
                    }
                }
            }
        }

        frame.scale_amp(volume.to_sample())
    }
}

impl<'a, FRM, NF, W, A, F, FW> Iterator for Frames<'a, FRM, NF, W, A, F, FW>
    where FRM: Frame,
          <FRM::Sample as Sample>::Float: sample::FromSample<f32>,
          <FRM::Sample as Sample>::Signed: sample::FromSample<f32>,
          NF: NoteFreq,
          W: Waveform,
          A: Amplitude,
          F: Frequency,
          FW: FreqWarp,
{
    type Item = FRM;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_frame())
    }
}
