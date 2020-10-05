use std::thread;
use std::time::Duration;

use dev_helpers::AudioEngine;
use dev_helpers::AudioProcessor;

use crossbeam_queue::spsc;

use mpm_pitch::PitchDetector;
use mpm_pitch::ProcessingResult;

fn note_number_to_string(note_number: f32) -> String {
    let note_names = [
        "A    ",
        "A#/B♭",
        "B    ",
        "C    ",
        "C#/D♭",
        "D    ",
        "D#/E♭",
        "E    ",
        "F    ",
        "F#/G♭",
        "G    ",
        "G#/A♭"
    ];
    let a0_number = 21;
    let nearest_midi_note = (note_number.round() as usize).max(a0_number);
    let octave_index = (nearest_midi_note - a0_number) / 12;
    let note_in_octave = (nearest_midi_note - a0_number) - 12 * octave_index;
    let cent_offset = note_number - (nearest_midi_note as f32);
    let cent_sign = if cent_offset > 0. { "+" } else { "-" };
    return format!("{}-{} | {}{:.2} cents", note_names[note_in_octave], octave_index, cent_sign, cent_offset.abs())
}

struct PitchReading {
    note_number: f32,
    frequency: f32,
}

struct MPMAudioProcessor {
    pitch_detector: PitchDetector,
}

impl MPMAudioProcessor {
    fn new(sample_rate: f32) -> MPMAudioProcessor {
        MPMAudioProcessor {
            pitch_detector: PitchDetector::new(sample_rate, 1024, 3 * 256, false),
        }
    }
}

impl AudioProcessor<PitchReading> for MPMAudioProcessor {
    fn process(
        &mut self,
        in_buffer: &[f32],
        _: &mut [f32],
        _: usize,
        to_main_thread: &spsc::Producer<PitchReading>,
        _: &spsc::Consumer<PitchReading>,
    ) -> bool {
        let mut sample_offset: usize = 0;
        while sample_offset < in_buffer.len() {
            match self.pitch_detector.process(&in_buffer[..], sample_offset) {
                ProcessingResult::ProcessedWindow { sample_index } => {
                    let result = &self.pitch_detector.result;
                    if result.is_valid() {
                        let push_result = to_main_thread.push(PitchReading {
                            note_number: result.note_number,
                            frequency: result.frequency
                        });

                    }
                    sample_offset = sample_index;
                }
                _ => {
                    break;
                }
            }
        }

        true
    }
}

fn main() {
    // Create an instance of an audio processor that does pitch detection on input samples
    let sample_rate = 44100.0;
    let processor = MPMAudioProcessor::new(sample_rate);
    // Create an audio engine that provides the processor with real time input samples
    let audio_engine = AudioEngine::new(sample_rate, processor);
    println!("Started audio engine");

    let poll_interval_ms = 30;

    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        loop {
            match audio_engine.from_audio_thread.pop() {
                Err(reason) => {
                    // println!("Failed to pop message from audio thread. {}", reason);
                    break;
                }
                Ok(reading) => {
                    println!("{} | {:.2} Hz", note_number_to_string(reading.note_number), reading.frequency)
                },
            }
        }
    }
}
