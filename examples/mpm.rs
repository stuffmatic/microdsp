use std::thread;
use std::time::Duration;

use dev_helpers::note_number_to_string;
use dev_helpers::AudioHost;
use dev_helpers::AudioProcessor;

use microdsp::mpm::PitchDetector;

struct PitchReading {
    midi_note_number: f32,
    frequency: f32,
}

struct MPMAudioProcessor {
    pitch_detector: PitchDetector,
}

impl MPMAudioProcessor {
    fn new(sample_rate: f32) -> MPMAudioProcessor {
        MPMAudioProcessor {
            pitch_detector: PitchDetector::new(sample_rate, 1024, 3 * 256),
        }
    }
}

impl AudioProcessor<PitchReading> for MPMAudioProcessor {
    fn process(
        &mut self,
        in_buffer: &[f32],
        _: &mut [f32],
        _: usize,
        to_main_thread: &mut dev_helpers::rtrb::Producer<PitchReading>,
        _: &mut dev_helpers::rtrb::Consumer<PitchReading>,
    ) -> bool {
        self.pitch_detector.process(in_buffer, |result| {
            if result.is_tone() {
                let _ = to_main_thread.push(PitchReading {
                    midi_note_number: result.midi_note_number,
                    frequency: result.frequency,
                });
            }
        });

        true
    }
}

fn main() {
    // Create an instance of an audio processor that does pitch detection on input samples
    let sample_rate = 44100.0;
    let processor = MPMAudioProcessor::new(sample_rate);
    // Create an audio engine that provides the audio processor with real time input samples
    let mut audio_engine = AudioHost::new(sample_rate, processor);
    println!("Started audio engine, listening for input. Whistle!");

    let poll_interval_ms = 30;

    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        loop {
            match audio_engine.from_audio_thread.pop() {
                Ok(reading) => println!(
                    "{} | {:.2} Hz",
                    note_number_to_string(reading.midi_note_number),
                    reading.frequency
                ),
                _ => break,
            }
        }
    }
}
