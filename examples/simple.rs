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
    // Construct the simplest possible synth --
    // a single voice containing a sine wave oscillating at 440 Hz.
    let mut synth = {
        use synth::{Synth, mode};

        // Create an oscillator tuned to C1 (32.70 Hz)
        let waveform = synth::oscillator::waveform::Sine;
        let amp = 1.0;
        let hz = 32.70;
        let freq_warp = ();
        let oscillator = synth::Oscillator::new(waveform, amp, hz, freq_warp);

        // Set up our synth using the oscillator and a single voice, based at C1
        Synth::new(mode::Mono::retrigger(), ())
            .oscillator(oscillator)
            .duration(6000.0)
            .base_pitch(LetterOctave(Letter::C, 1).hz())
            .fade(500.0, 500.0)
            .num_voices(1)
            .volume(0.20)
    };

    // Trigger the synth at A4 (440 Hz)
    let note = LetterOctave(Letter::A, 4);
    let note_velocity = 1.0;
    synth.note_on(note, note_velocity);

    // These variables are used to break the audio loop after four seconds
    let note_duration = 4.0;
    let mut is_note_off = false;
    let mut timer: f64 = 0.0;
    let mut prev_time = None;

    // This callback gets passed to our stream
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, time, .. }| {
        dsp::sample::buffer::equilibrium(buffer);
        let settings = Settings::new(SAMPLE_HZ as u32, frames as u16, CHANNELS as u16);
        synth.audio_requested(buffer, settings);
        if timer < 6.0 {
            let last_time = prev_time.unwrap_or(time.current);
            let dt = time.current - last_time;
            timer += dt;
            prev_time = Some(time.current);

            // Break if we've exceeded the desired note duration
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

    // Set up PortAudio and the stream
    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Loop while the stream is active
    while let Ok(true) = stream.is_active() {}

    Ok(())
}
