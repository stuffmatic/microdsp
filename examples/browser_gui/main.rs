mod audio;
use std::thread;
use std::time::Duration;
use crossbeam_queue::spsc;
use mpm_pitch::pitch_detection_result::PitchDetectionResult;
use mpm_pitch::pitch_detector::PitchDetector;
use mpm_pitch::pitch_detector::ProcessingResult;

enum MPMAudioProcessorMessage {
    DetectedPitch {
        timestamp: f32,
        frequency: f32,
        clarity: f32,
        note_number: f32,
        window_rms: f32,
        window_rms_db: f32,
        window_peak: f32,
        window_peak_db: f32,
    },
}

struct MPMAudioProcessor {
    to_main_thread: spsc::Producer<MPMAudioProcessorMessage>,
    from_main_thread: spsc::Consumer<MPMAudioProcessorMessage>,
    processed_sample_count: usize,
    sample_rate: f32,
    pitch_detector: PitchDetector,
}

impl MPMAudioProcessor {
    fn new(
        sample_rate: f32,
        to_main_thread: spsc::Producer<MPMAudioProcessorMessage>,
        from_main_thread: spsc::Consumer<MPMAudioProcessorMessage>,
    ) -> MPMAudioProcessor {
        MPMAudioProcessor {
            processed_sample_count: 0,
            sample_rate,
            pitch_detector: PitchDetector::new(sample_rate, 512, 256, false),
            to_main_thread,
            from_main_thread,
        }
    }
}

impl audio::AudioProcessor<MPMAudioProcessorMessage> for MPMAudioProcessor {
    fn process(&mut self, in_buffer: &[f32], out_buffer: &mut [f32], frame_count: usize) -> bool {
        let mut sample_offset: usize = 0;
        while sample_offset < in_buffer.len() {
            match self.pitch_detector.process(&in_buffer[..], sample_offset) {
                ProcessingResult::ProcessedWindow { sample_index } => {
                    let timestamp =
                        ((self.processed_sample_count + sample_offset) as f32) / self.sample_rate;
                    let result = &self.pitch_detector.result;
                    let push_result =
                        self.to_main_thread
                            .push(MPMAudioProcessorMessage::DetectedPitch {
                                timestamp,
                                frequency: result.frequency,
                                clarity: result.clarity,
                                note_number: result.note_number,
                                window_rms: result.window_rms(),
                                window_rms_db: result.window_rms_db(),
                                window_peak: result.window_peak(),
                                window_peak_db: result.window_peak_db(),
                            });
                    sample_offset = sample_index;
                }
                _ => {
                    break;
                }
            }
        }
        self.processed_sample_count += in_buffer.len();

        true
    }
}

fn main() {
    let sample_rate = 44100.0;
    let queue_capacity = 1000;
    let (to_audio_thread, from_main_thread) = spsc::new::<MPMAudioProcessorMessage>(queue_capacity);
    let (to_main_thread, from_audio_thread) = spsc::new::<MPMAudioProcessorMessage>(queue_capacity);
    let processor = MPMAudioProcessor::new(sample_rate, to_main_thread, from_main_thread);

    println!("Starting audio thread");
    let stream = audio::run_processor(processor);

    let poll_interval_ms = 30;
    println!("Entering event loop, polling every {} ms", poll_interval_ms);
    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        loop {
            match from_audio_thread.pop() {
                Err(reason) => {
                    // println!("Failed to pop {} on audio thread", reason);
                    break;
                }
                Ok(message) => match message {
                    MPMAudioProcessorMessage::DetectedPitch {
                      timestamp,
                      frequency,
                      clarity,
                      note_number,
                      window_rms,
                      window_rms_db,
                      window_peak,
                      window_peak_db
                    } => {
                      println!("DetectedPitch: {} Hz, clarity {}, RMS {} dB", frequency, clarity, window_rms_db)
                    }
                },
            }
        }

        //let f = rng.gen_range(300.0, 1000.0);
        /*
        match engine.to_audio_thread.push(TestSynthMessage::PlayNote(f)) {
            Err(reason) => println!("Failed to push message"),
            Ok(_) => println!("Sent lol from main thread")
        }*/
    }
}
