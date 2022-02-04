use microear::{
    common::window_function::HannWindow,
    snov::{compression_function::HardKneeCompression, detector::SpectralNoveltyDetector},
};
use dev_helpers::portaudio as pa;
use dev_helpers::rtrb;
use std::thread;
use std::time::Duration;

pub trait AudioProcessor<S> {
    /// Return false to stop the audio stream, true otherwise.
    fn process(
        &mut self,
        in_buffer: &[f32],
        out_buffer: &mut [f32],
        frame_count: usize,
        to_main_thread: &mut rtrb::Producer<S>,
        from_main_thread: &mut rtrb::Consumer<S>,
    ) -> bool;
}

pub struct AudioEngine<S> {
    pa_stream: pa::Stream<pa::NonBlocking, pa::Duplex<f32, f32>>,
    pub to_audio_thread: rtrb::Producer<S>,
    pub from_audio_thread: rtrb::Consumer<S>,
}

impl<S> AudioEngine<S>
where
    S: 'static,
{
    pub fn new<T: AudioProcessor<S> + 'static>(sample_rate: f32, mut processor: T) -> Self {
        let queue_capacity = 1000;
        let (to_audio_thread, mut from_main_thread) =
            rtrb::RingBuffer::<S>::new(queue_capacity).split();
        let (mut to_main_thread, from_audio_thread) =
            rtrb::RingBuffer::<S>::new(queue_capacity).split();
        let pa = pa::PortAudio::new().unwrap();
        let default_input = pa.default_input_device().unwrap();
        let default_output = pa.default_output_device().unwrap();
        let input_info = pa.device_info(default_input).unwrap();
        println!("Using audio input device \"{}\"", input_info.name);

        let latency = input_info.default_low_input_latency;
        let input_params = pa::StreamParameters::<f32>::new(default_input, 1, true, latency);
        let output_params = pa::StreamParameters::new(default_output, 1, true, latency);
        let settings =
            pa::DuplexStreamSettings::new(input_params, output_params, sample_rate as f64, 256);

        let pa_callback = move |pa::DuplexStreamCallbackArgs {
                                    in_buffer,
                                    out_buffer,
                                    frames,
                                    time: _,
                                    ..
                                }| {
            match processor.process(
                in_buffer,
                out_buffer,
                frames,
                &mut to_main_thread,
                &mut from_main_thread,
            ) {
                true => pa::Continue,
                false => pa::Complete,
            }
        };
        let mut stream = pa.open_non_blocking_stream(settings, pa_callback).unwrap();
        stream.start().unwrap();
        AudioEngine {
            pa_stream: stream,
            to_audio_thread,
            from_audio_thread,
        }
    }

    pub fn stop(&mut self) {
        self.pa_stream.stop().unwrap()
    }
}

enum DetectorMessage {
    NoveltyValue(f32),
}

const WINDOW_SIZE: usize = 1024;

type DetectorType = SpectralNoveltyDetector<HannWindow, HardKneeCompression>;
struct NoveltyDetectorProcessor {
    detector: DetectorType,
}

impl NoveltyDetectorProcessor {
    fn new() -> Self {
        NoveltyDetectorProcessor {
            detector: DetectorType::new(WINDOW_SIZE),
        }
    }
}

impl AudioProcessor<DetectorMessage> for NoveltyDetectorProcessor {
    fn process(
        &mut self,
        in_buffer: &[f32],
        _: &mut [f32],
        _: usize,
        to_main_thread: &mut rtrb::Producer<DetectorMessage>,
        _: &mut rtrb::Consumer<DetectorMessage>,
    ) -> bool {
        self.detector.process(in_buffer, |novelty| {
            let _ = to_main_thread.push(DetectorMessage::NoveltyValue(novelty.novelty()));
        });
        true
    }
}

fn main() {
    // Create an instance of an audio processor that does pitch detection on input samples
    let sample_rate = 44100.0;
    let processor = NoveltyDetectorProcessor::new();
    // Create an audio engine that provides the audio processor with real time input samples
    let mut audio_engine = AudioEngine::new(sample_rate, processor);
    println!("Started audio engine, listening for input.");

    let poll_interval_ms = 30;
    let mut is_armed = true;
    let threshold = 0.4;

    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        loop {
            match audio_engine.from_audio_thread.pop() {
                Ok(message) => match message {
                    DetectorMessage::NoveltyValue(value) => {
                        if value > threshold && is_armed {
                            is_armed = false;
                            println!("Novelty above threshold {}", value)
                        } else if value < threshold {
                            is_armed = true;
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
