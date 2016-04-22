//!
//!  simple.rs
//!
//!  John Berry <ulfmagnetics@gmail.com>
//!
//!  Always remember to run high performance Rust code with the --release flag. `Synth`
//!

extern crate dsp;
extern crate pitch_calc as pitch;
extern crate portaudio;
extern crate synth;

use dsp::{Node, Settings};
use pitch::{Letter, LetterOctave};
use portaudio as pa;

const CHANNELS: i32 = 2;
const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;

fn main() {
  run().unwrap()
}

fn run() -> Result<(), pa::Error> {
    // Construct the simplest possible synth
    let mut synth = {
        use synth::dynamic::{Oscillator, Waveform, Amplitude, Frequency, FreqWarp};

        let oscillator = Oscillator::new(Waveform::Sine,
            Amplitude::Constant(1.0),
            Frequency::Hz(440.0),
            FreqWarp::None);

        synth::dynamic::new()
            .oscillator(oscillator)
            .duration(6000.0)
            .fade(500.0, 500.0)
            .num_voices(1)
            .volume(0.20)
    };

    let note = LetterOctave(Letter::C, 1);
    let note_velocity = 1.0;
    synth.note_on(note, note_velocity);

    let note_duration = 4.0;
    let mut is_note_off = false;
    let mut timer: f64 = 0.0;
    let mut prev_time = None;

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, time, .. }| {
        dsp::sample::buffer::equilibrium(buffer);
        let settings = Settings::new(SAMPLE_HZ as u32, frames as u16, CHANNELS as u16);
        synth.audio_requested(buffer, settings);
        if timer < 6.0 {
            let last_time = prev_time.unwrap_or(time.current);
            let dt = time.current - last_time;
            timer += dt;
            prev_time = Some(time.current);

            if timer > note_duration {
                if !is_note_off {
                    synth.note_off(note);
                    is_note_off = true;
                }
            }
            pa::Continue
        } else {
            pa::Complete
        }
    };

    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    while let Ok(true) = stream.is_active() {}

    Ok(())
}
