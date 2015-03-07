//!
//!  synth_voice.rs
//!
//!  Created by Mitchell Nordine at 04:01PM on June 28, 2014.
//!
//!

use dsp::Settings as DspSettings;
use dsp::{Sample};
use oscillator::Oscillator;
use time::{self, Samples};
use envelope::{Envelope, Point};
use waveform::Waveform;

pub type Playhead = time::calc::Samples;
pub type LoopStart = time::calc::Samples;
pub type LoopEnd = time::calc::Samples;
pub type Attack = time::calc::Samples;
pub type Release = time::calc::Samples;
pub type LoopPlayhead = time::calc::Samples;
pub type NoteDuration = time::calc::Samples;
pub type NoteFreqMulti = f64;

/// A single Voice. A Synth may consist
/// of any number of Voices.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Voice {
    /// Oscillators for playback.
    pub oscillators: Vec<Oscillator>,
    /// Data for a note, if there is one currently being played.
    pub maybe_note: Option<(NoteDuration, NoteFreqMulti)>,
    /// Playhead over the current note.
    pub playhead: Playhead,
    /// Playhead over the loop duration.
    pub loop_playhead: LoopPlayhead,
}

impl Voice {

    /// Constructor for a Voice.
    pub fn new(oscillators: Vec<Oscillator>) -> Voice {
        Voice {
            oscillators: oscillators,
            maybe_note: None,
            playhead: 0,
            loop_playhead: 0,
        }
    }

    /// Default constructor for a Voice with a single Oscillator.
    pub fn default() -> Voice {
        Voice::new(vec!(Oscillator::new()))
    }

    /// Testing constructor. Creates a basic Kick sound.
    pub fn test_demo() -> Voice {
        let amp_env = Envelope::from_points(vec!(
            Point::new(0.0,  0.0, 0.0),
            Point::new(0.01, 1.0, 0.0),
            Point::new(0.45, 1.0, 0.0),
            Point::new(0.81, 0.8, 0.0),
            Point::new(1.0,  0.0, 0.0),
        ));
        let freq_env = Envelope::from_points(vec!(
            Point::new(0.0,     0.0,    0.0),
            Point::new(0.00136, 1.0   , 0.0),
            Point::new(0.015  , 0.01  , 0.0),
            Point::new(0.045  , 0.005 , 0.0),
            Point::new(0.1    , 0.0022, 0.0),
            Point::new(0.35   , 0.0011, 0.0),
            Point::new(1.0,     0.0,    0.0),
        ));
        let oscillator = Oscillator::new()
            .waveform(Waveform::Sine)
            .amplitude(amp_env)
            .frequency(freq_env);

        Voice::new(vec!(oscillator))
    }

    /// Trigger playback with the given note, resetting all playheads.
    #[inline]
    pub fn play_note(&mut self, note: (NoteDuration, NoteFreqMulti)) {
        self.maybe_note = Some(note);
        self.playhead = 0;
        self.loop_playhead = 0;
    }

    /// Generate and fill the audio buffer for the given parameters.
    #[inline]
    pub fn fill_buffer<S>(&mut self,
                          output: &mut [S],
                          settings: DspSettings,
                          duration: time::calc::Samples,
                          loop_data: Option<&(LoopStart, LoopEnd)>,
                          fade_data: Option<&(Attack, Release)>) where S: Sample {
        let Voice {
            ref mut oscillators,
            ref mut playhead,
            ref mut loop_playhead,
            ref mut maybe_note,
        } = *self;
        let (attack, release) = fade_data.map_or_else(|| (0, 0), |&(a, r)| (a, r));
        let (note_duration, note_freq_multi) = maybe_note.unwrap_or_else(||(duration, 1.0));

        for frame in output.chunks_mut(settings.channels as usize) {

            // Calculate the amplitude of the current frames.
            let wave = match (maybe_note.is_some(), *loop_playhead < duration) {
                (true, true) => {
                    let ratio = *loop_playhead as f64 / duration as f64;
                    // Sum the amplitude of each oscillator at the given ratio.
                    oscillators.iter_mut().fold(0.0, |total, osc| {
                        let mut wave = osc.amp_at_ratio(ratio,
                                                        note_freq_multi,
                                                        settings.sample_hz as f64);
                        // If within the attack duration, apply the fade.
                        if *playhead < attack {
                            wave *= *playhead as f32 / attack as f32;
                        }
                        // If within the release duration, apply the fade.
                        if *playhead > note_duration && release > 0 {
                            wave *= (release - (*playhead - note_duration)) as f32 / release as f32;
                        }
                        wave + total
                    })
                },
                // If the playhead is out of range or if no note was given, zero the buffer.
                _ => 0.0,
            };

            // Assign the amp to each channel.
            for channel in frame.iter_mut() {
                *channel = Sample::from_wave(wave);
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
            if *playhead >= note_duration + release || *loop_playhead > duration {
                *maybe_note = None;
                *playhead = 0;
            }

        }
    }

}

