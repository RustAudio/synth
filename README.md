# synth [![Build Status](https://travis-ci.org/RustAudio/synth.svg?branch=master)](https://travis-ci.org/RustAudio/synth) [![Crates.io](https://img.shields.io/crates/v/synth.svg)](https://crates.io/crates/synth) [![Crates.io](https://img.shields.io/crates/l/synth.svg)](https://github.com/RustAudio/synth/blob/master/LICENSE)


A polyphonic Synth type whose multiple oscillators generate sound via amplitude and frequency envelopes.

Features
--------

- Sine, Saw, SawExp, Square, Noise and NoiseWalk waveforms.
- Amplitude and frequency envelopes with an unlimited number of points.
- Unlimited number of oscillators (each can have unique waveforms and amplitude and frequency envelopes).
- Monophonic and Polyphonic modes (unlimited number of voices).
- Simple `note_on(pitch_in_hz, velocity)` and `note_off(pitch_in_hz)` methods.
- Per-channel amplitude and a stereo panning helper method.
- "Stereo spread" for automatically spreading multiple voices evenly across the stereo image.
- Per-voice portamento.
- Per-voice detuning.
- Multi-voice (unison) support in Mono mode.
- Legato and Retrigger Mono modes.
- Warbliness Oscillator builder method that uses gaussian noise to model the "warped-old-hardware-synth" sound.

```Rust
synth.fill_slice(frame_slice, sample_hz),
```

See an example [here](https://github.com/RustAudio/synth/blob/master/examples/test.rs).
