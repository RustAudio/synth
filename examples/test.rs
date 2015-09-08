//!
//!  test.rs.rs
//!
//!  Created by Mitchell Nordine at 05:57PM on December 19, 2014.
//!
//!  Always remember to run high performance Rust code with the --release flag. `Synth` 
//!

extern crate dsp;
extern crate pitch_calc as pitch;
extern crate synth;
extern crate time_calc as time;

use dsp::{CallbackFlags, CallbackResult, Node, Sample, SoundStream, Settings, StreamParams};
use pitch::{Letter, LetterOctave};
use synth::Synth;

// Currently supports i8, i32, f32.
pub type AudioSample = f32;
pub type Input = AudioSample;
pub type Output = AudioSample;

fn main() {

    // Construct our fancy Synth!
    let mut synth = {
        use synth::{Point, Oscillator, mode, oscillator};

        // The following envelopes should create a downward pitching sine wave that gradually quietens.
        // Try messing around with the points and adding some of your own!
        let amp_env = oscillator::AmpEnvelope::from_points(vec!(
            //         Time ,  Amp ,  Curve
            Point::new(0.0  ,  0.0 ,  0.0),
            Point::new(0.01 ,  1.0 ,  0.0),
            Point::new(0.45 ,  1.0 ,  0.0),
            Point::new(0.81 ,  0.8 ,  0.0),
            Point::new(1.0  ,  0.0 ,  0.0),
        ));
        let freq_env = oscillator::FreqEnvelope::from_points(vec!(
            //         Time    , Freq   , Curve
            Point::new(0.0     , 0.0    , 0.0),
            Point::new(0.00136 , 1.0    , 0.0),
            Point::new(0.015   , 0.02   , 0.0),
            Point::new(0.045   , 0.005  , 0.0),
            Point::new(0.1     , 0.0022 , 0.0),
            Point::new(0.35    , 0.0011 , 0.0),
            Point::new(1.0     , 0.0    , 0.0),
        ));

        // Now we can create our oscillator from our envelopes.
        // There are also Sine, Noise, NoiseWalk, SawExp and Square waveforms.
        let oscillator = Oscillator::new(oscillator::waveform::Square, amp_env, freq_env, ());

        // Here we construct our Synth from our oscillator.
        Synth::new(mode::Mono::retrigger(), ())
            .oscillator(oscillator) // Add as many different oscillators as desired.
            .duration(6000.0) // Milliseconds.
            .base_pitch(LetterOctave(Letter::C, 1).hz()) // Hz.
            .loop_points(0.49, 0.51) // Loop start and end points.
            .fade(500.0, 500.0) // Attack and Release in milliseconds.
            .num_voices(16) // By default Synth is monophonic but this gives it `n` voice polyphony.
            .volume(0.20)
            .detune(0.5)
            .spread(1.0)

        // Other methods include:
            // .loop_start(0.0)
            // .loop_end(1.0)
            // .attack(ms)
            // .release(ms)
            // .note_freq_generator(nfg)
            // .oscillators([oscA, oscB, oscC])
            // .volume(1.0)
    };

    // Construct a note for the synth to perform. Have a play around with the pitch and duration!
    let note = LetterOctave(Letter::C, 1);
    let note_velocity = 1.0;
    synth.note_on(note, note_velocity);

    // We'll call this to release the note after 4 seconds.
    let note_duration = 4.0;
    let mut is_note_off = false;

    // We'll use this to keep track of time and break from the loop after 6 seconds.
    let mut timer: f64 = 0.0;

    // The callback we'll use to pass to the Stream.
    let callback = Box::new(move |output: &mut[f32], settings: Settings, dt: f64, _: CallbackFlags| {
        Sample::zero_buffer(output);
        synth.audio_requested(output, settings);
        if timer < 6.0 {
            timer += dt;
            // Once the timer exceeds our note duration, send the note_off.
            if timer > note_duration {
                if !is_note_off {
                    synth.note_off(note);
                    is_note_off = true;
                }
            }
            CallbackResult::Continue
        } else {
            CallbackResult::Complete
        }
    });

    // Construct the default, non-blocking output stream and run our callback.
    let stream = SoundStream::new().output(StreamParams::new()).run_callback(callback).unwrap();

    // Loop while the stream is active.
    while let Ok(true) = stream.is_active() {}

}
