extern crate dsp;

use {Synth, instrument, oscillator};
use self::dsp::Sample;

impl<FRM, M, NFG, W, A, F, FW> dsp::Node<FRM> for Synth<M, NFG, W, A, F, FW>
    where FRM: dsp::Frame,
          <FRM::Sample as Sample>::Float: dsp::FromSample<f32>,
          <FRM::Sample as Sample>::Signed: dsp::FromSample<f32>,
          M: instrument::Mode,
          NFG: instrument::NoteFreqGenerator,
          W: oscillator::Waveform,
          A: oscillator::Amplitude,
          F: oscillator::Frequency,
          FW: oscillator::FreqWarp,
{
    #[inline]
    fn audio_requested(&mut self, output: &mut [FRM], sample_hz: f64) {
        self.fill_slice(output, sample_hz);
    }
}
