use dev_helpers::rtrb;
use dev_helpers::{AudioHost, AudioProcessor};
use microdsp::common::window::Window;
use microdsp::snov::{
    compression_function::HardKneeCompression, spectral_flux_novelty_detector::SpectralFluxNoveltyDetector,
};
use std::thread;
use std::time::Duration;

enum DetectorMessage {
    NoveltyValue(f32),
    PeakValue(f32),
}

const WINDOW_SIZE: usize = 1024;

type DetectorType = SpectralFluxNoveltyDetector<HardKneeCompression>;
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

        let _ = to_main_thread.push(DetectorMessage::PeakValue(in_buffer.peak_level()));
        true
    }
}

struct PeakDetector {
    threshold: f32,
    is_armed: bool,
}

impl PeakDetector {
    fn new(threshold: f32) -> Self {
        PeakDetector {
            threshold,
            is_armed: true
        }
    }

    fn process(&mut self, input: f32) -> bool {
        if input > self.threshold && self.is_armed {
            self.is_armed = false;
            return true
        } else if input < self.threshold {
            self.is_armed = true;
        };
        false
    }
}

fn main() {
    let sample_rate = 44100.0;
    let mut audio_host = AudioHost::new(
        sample_rate,
        NoveltyDetectorProcessor::new()
    );
    println!("Listening for input...");

    let poll_interval_ms = 30;
    let mut novelty_peak_detector = PeakDetector::new(0.4);
    let mut naive_peak_detector = PeakDetector::new(0.4);

    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        loop {
            match audio_host.from_audio_thread.pop() {
                Ok(message) => match message {
                    DetectorMessage::NoveltyValue(value) => {
                        if novelty_peak_detector.process(value) {
                            println!("Novelty peak detected {}", value)
                        }
                    },
                    DetectorMessage::PeakValue(value) => {
                        if naive_peak_detector.process(value) {
                            println!("Naive peak detected {}", value)
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
