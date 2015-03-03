# synth [![Build Status](https://travis-ci.org/RustAudio/synth.svg?branch=master)](https://travis-ci.org/RustAudio/synth)

A polyphonic Synth type whose multiple oscillators generate sound via amplitude and frequency envelopes.

Features
--------

- Sine, Saw, Square, Noise and NoiseWalk waveforms.
- Amplitude and frequency envelopes with an unlimited number of points.
- Unlimited number of oscillators (each can have unique waveforms and amplitude and frequency envelopes).
- Polyphonic (unlimited number of voices).
- Warbliness builder method that uses gaussian noise to model the "warped-old-hardware-synth" sound.
- Trigger playback at any time by passing the synth notes `synth.play_note((duration_in_ms, pitch_in_hz))`. `Synth` will automatically find an idle `Voice` to use (or retrigger the oldest one).
- Uses [sound_stream](https://github.com/RustAudio/sound_stream) and its Sample trait and in turn is generic over any bit-depth or sample format.

```Rust
for event in stream.by_ref() {
    match event {
        Event::Out(buffer) => synth.audio_requested(buffer, SETTINGS),
        ..
    }
}
```

See an example [here](https://github.com/RustAudio/synth/blob/master/examples/test.rs).

PortAudio
---------

At the moment you need to have [PortAudio](http://www.portaudio.com/download.html) installed on your system, but the rust-portaudio crate should take care of this automatically with it's cargo build script.
