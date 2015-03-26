//!
//!  test.rs.rs
//!
//!  Created by Mitchell Nordine at 05:57PM on December 19, 2014.
//!
//!

extern crate dsp;
extern crate pitch_calc as pitch;
extern crate synth;
extern crate time_calc as time;

use dsp::{Event, Node, Settings, SoundStream};
use pitch::{Letter, LetterOctave};
use synth::Synth;

const SETTINGS: Settings = Settings { sample_hz: 44_100, frames: 256, channels: 2 };

// Currently supports i8, i32, f32.
pub type AudioSample = f32;
pub type Input = AudioSample;
pub type Output = AudioSample;

fn main() {

    // Construct the stream and handle any errors that may have occurred.
    let mut stream = match SoundStream::<Input, Output>::new().settings(SETTINGS).run() {
        Ok(stream) => { println!("It begins!"); stream },
        Err(err) => panic!("An error occurred while constructing SoundStream: {}", err),
    };

    // Construct our fancy Synth!
    let mut synth = {
        use synth::{AmpEnvelope, FreqEnvelope, Point, Oscillator, Waveform};

        // The following envelopes should create a downward pitching sine wave that gradually quietens.
        // Try messing around with the points and adding some of your own!
        let amp_env = AmpEnvelope::from_points(vec!(
            //         Time ,  Amp ,  Curve
            Point::new(0.0  ,  0.0 ,  0.0),
            Point::new(0.01 ,  1.0 ,  0.0),
            Point::new(0.45 ,  1.0 ,  0.0),
            Point::new(0.81 ,  0.8 ,  0.0),
            Point::new(1.0  ,  0.0 ,  0.0),
        ));
        let freq_env = FreqEnvelope::from_points(vec!(
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
            .waveform(Waveform::Sine) // There are also Saw, Noise, NoiseWalk, SawExp and Square waveforms.
            .amplitude(amp_env)
            .frequency(freq_env);

        // Here we construct our Synth from our oscillator.
        Synth::new()
            .oscillator(oscillator) // Add as many different oscillators as desired.
            .duration(4000.0) // Milliseconds.
            .base_pitch(LetterOctave(Letter::C, 1).hz()) // Hz.
            .loop_points(0.49, 0.51) // Loop start and end points.
            .fade(500.0, 500.0) // Attack and Release in milliseconds.
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
    let note_hz = LetterOctave(Letter::C, 1).hz();
    let note_velocity = 1.0;
    println!("note_on");
    synth.note_on(note_hz, note_velocity);

    // We'll call this to release the note after 2 seconds.
    let release_time = 2.0;
    let mut maybe_note_off = Some(|synth: &mut Synth| synth.note_off(note_hz));

    // We'll use this to keep track of time and break from the loop after 5 seconds.
    let mut timer: f64 = 0.0;

    // The SoundStream iterator will automatically return these events in this order.
    for event in stream.by_ref() {
        match event {
            Event::Out(buffer, settings) => synth.audio_requested(buffer, settings),
            Event::Update(dt) => if timer < 5.0 {
                timer += dt;
                if timer > release_time {
                    if let Some(note_off) = maybe_note_off.take() {
                        println!("note_off");
                        note_off(&mut synth);
                    }
                }
            } else { break },
            _ => (),
        }
    }

    // Close the stream and shut down PortAudio.
    match stream.close() {
        Ok(()) => println!("Great success!"),
        Err(err) => println!("An error occurred while closing SoundStream: {}", err),
    }

}
