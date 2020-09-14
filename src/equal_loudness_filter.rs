// https://en.wikipedia.org/wiki/Infinite_impulse_response
pub (crate) struct IIRFilter {
  a_coeffs: Vec<f32>,
  b_coeffs: Vec<f32>,
  inputs: Vec<f32>,
  inputs_pos: usize,
  outputs: Vec<f32>,
  outputs_pos: usize,
}

impl IIRFilter {
  pub (crate) fn new(a_coeffs: Vec<f32>, b_coeffs: Vec<f32>) -> IIRFilter {
      let inputs_count = b_coeffs.len();
      let outputs_count = a_coeffs.len();
      IIRFilter {
          a_coeffs,
          b_coeffs,
          inputs: vec![0.0; inputs_count],
          inputs_pos: 0,
          outputs: vec![0.0; outputs_count],
          outputs_pos: 0,
      }
  }

  pub (crate) fn process(&mut self, input_samples: &[f32], output_samples: &mut [f32]) {
      if input_samples.len() != output_samples.len() {
          panic!("IIR filter input and output buffers must have the same size");
      }
      for (index, input) in input_samples.iter().enumerate() {
          output_samples[index] = input_samples[index];
      }
  }
}

pub (crate) struct EqualLoudnessFilter {
  butterworth: IIRFilter,
  yule_walk: IIRFilter,
}

impl EqualLoudnessFilter {
  pub (crate) fn new(sample_rate: f32) -> EqualLoudnessFilter {
      if sample_rate as usize != 44100 {
          panic!("Only a sample rate of 44100 Hz is supported")
      }
      EqualLoudnessFilter {
          butterworth: IIRFilter::new(
              vec![1.00000000000000, -1.96977855582618, 0.97022847566350],
              vec![0.98500175787242, -1.97000351574484, 0.98500175787242],
          ),
          yule_walk: IIRFilter::new(
              vec![
                  1.00000000000000,
                  -3.47845948550071,
                  6.36317777566148,
                  -8.54751527471874,
                  9.47693607801280,
                  -8.81498681370155,
                  6.85401540936998,
                  -4.39470996079559,
                  2.19611684890774,
                  -0.75104302451432,
                  0.13149317958808,
              ],
              vec![
                  0.05418656406430,
                  -0.02911007808948,
                  -0.00848709379851,
                  -0.00851165645469,
                  -0.00834990904936,
                  0.02245293253339,
                  -0.02596338512915,
                  0.01624864962975,
                  -0.00240879051584,
                  0.00674613682247,
                  -0.00187763777362,
              ],
          ),
      }
  }

  pub (crate) fn process(&mut self, input: &[f32], output: &mut [f32]) {
      self.yule_walk.process(input, output);
      self.butterworth.process(input, output);
  }
}