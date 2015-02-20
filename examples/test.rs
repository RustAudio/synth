//!
//!  test.rs.rs
//!
//!  Created by Mitchell Nordine at 05:57PM on December 19, 2014.
//!
//!

extern crate dsp;
extern crate envelope;
extern crate "pitch_calc" as pitch;
extern crate synth;
extern crate "time_calc" as time;

use dsp::{Event, Node, Settings, SoundStream};
use pitch::{Letter, LetterOctave};
use time::Ms;

const SETTINGS: Settings = Settings { sample_hz: 44_100, frames: 256, channels: 2 };

// Currently supports i8, u8, i32, f32.
pub type AudioSample = f32;
pub type Input = AudioSample;
pub type Output = AudioSample;
pub type OutputBuffer = [Output; SETTINGS.frames as usize * SETTINGS.channels as usize];

fn main() {

    // Construct the stream and handle any errors that may have occurred.
    let mut stream = match SoundStream::<OutputBuffer, Input>::new(SETTINGS) {
        Ok(stream) => { println!("It begins!"); stream },
        Err(err) => panic!("An error occurred while constructing SoundStream: {}", err),
    };

    // Construct our fancy Synth!
    let mut synth = {
        use envelope::{Envelope, Point};
        use synth::{Oscillator, Synth, Waveform};

        // The following envelopes should create a downward pitching sine wave that gradually quietens.
        // Try messing around with the points and adding some of your own!
        let amp_env = Envelope::from_points(vec!(
            //         Time ,  Amp ,  Curve
            Point::new(0.0  ,  0.0 ,  0.0),
            Point::new(0.01 ,  1.0 ,  0.0),
            Point::new(0.45 ,  1.0 ,  0.0),
            Point::new(0.81 ,  0.8 ,  0.0),
            Point::new(1.0  ,  0.0 ,  0.0),
        ));
        let freq_env = Envelope::from_points(vec!(
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
        let oscillator = Oscillator::new()
            .waveform(Waveform::Sine) // There are also Saw, Noise, NoiseWalk and Square waveforms.
            .amplitude(amp_env)
            .frequency(freq_env);

        // Here we construct our Synth from our oscillator.
        Synth::new()
            .oscillator(oscillator) // Add as many different oscillators as desired.
            .duration(4000.0) // Milliseconds.
            .base_pitch(LetterOctave(Letter::C, 1).hz()) // Hz.
            .loop_points(0.49, 0.51) // Loop start and end points.
            .fade(500.0, 500.0) // Attack and release.
            .num_voices(16) // By default Synth is monophonic but this gives it `n` voice polyphony.
        // Other methods include:
            // .loop_start(0.0)
            // .loop_end(1.0)
            // .attack(ms)
            // .release(ms)
            // .oscillators([oscA, oscB, oscC])
            // .volume(1.0)
            // .normaliser(1.0)
    };

    // Construct a note for the synth to perform. Have a play around with the pitch and duration!
    synth.play_note((
        Ms(2000.0).samples(SETTINGS.sample_hz as f64), // Note duration in samples.
        LetterOctave(Letter::C, 1).hz() // Note pitch in hz.
    ));

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // The SoundStream iterator will automatically return these events in this order.
    for event in stream.by_ref() {
        match event {
            Event::Out(buffer) => synth.audio_requested(buffer, SETTINGS),
            Event::Update(dt) => if timer > 0.0 { timer -= dt } else { break },
            _ => (),
        }
    }

    // Close the stream and shut down PortAudio.
    match stream.close() {
        Ok(()) => println!("Great success!"),
        Err(err) => println!("An error occurred while closing SoundStream: {}", err),
    }

}
