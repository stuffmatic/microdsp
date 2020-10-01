extern crate portaudio;
use portaudio as pa;

pub trait AudioProcessor<T> {
    /// Return false to stop the audio stream, true otherwise.
    fn process(&mut self, in_buffer: &[f32], out_buffer: &mut [f32], frame_count: usize) -> bool;
}

pub fn run_processor<S, T: AudioProcessor<S> + 'static>(
  mut processor: T,
) -> pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>> {
  let pa = pa::PortAudio::new().unwrap();
  let default_input = pa.default_input_device().unwrap();
  let default_output = pa.default_output_device().unwrap();
  let input_info = pa.device_info(default_input).unwrap();
  println!("Using audio input device \"{}\"", input_info.name);

  let latency = input_info.default_low_input_latency;
  let input_params = pa::StreamParameters::<f32>::new(default_input, 1, true, latency);
  let output_params = pa::StreamParameters::new(default_output, 1, true, latency);
  let settings = pa::DuplexStreamSettings::new(input_params, output_params, 44100.0, 256);

  let pa_callback = move |pa::DuplexStreamCallbackArgs {
                              in_buffer,
                              out_buffer,
                              frames,
                              time,
                              ..
                          }| {
      match processor.process(in_buffer, out_buffer, frames) {
        true => pa::Continue,
        false => pa::Complete
      }
  };
  let mut stream = pa.open_non_blocking_stream(settings, pa_callback).unwrap();
  stream.start().unwrap();
  stream
}
