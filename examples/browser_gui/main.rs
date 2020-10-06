use std::thread;
use std::time::Duration;

use serde::Serialize;
use serde_json;

use crossbeam_queue::spsc;

use dev_helpers::AudioEngine;
use dev_helpers::AudioProcessor;
use dev_helpers::WebsocketServer;
use dev_helpers::note_number_to_string;

use mpm_pitch::KeyMaximum;
use mpm_pitch::Result;
use mpm_pitch::Detector;
use mpm_pitch::ProcessingResult;

const MAX_NSDF_SIZE: usize = 1024;

#[derive(Copy, Clone, Serialize)]
pub struct PitchReadingKeyMax {
    pub lag_index: usize,
    pub value_at_lag_index: f32,
    pub value: f32,
    pub lag: f32,
}

impl PitchReadingKeyMax {
    fn new() -> PitchReadingKeyMax {
        PitchReadingKeyMax {
            lag_index: 0,
            lag: 0.,
            value_at_lag_index: 0.,
            value: 0.,
        }
    }
}

const MAX_KEY_MAXIMA_COUNT: usize = 64;

#[derive(Serialize)]
struct PitchReadingInfo {
    window_size: usize,
    timestamp: f32,
    frequency: f32,
    clarity: f32,
    note_number: f32,
    window_rms: f32,
    window_peak: f32,
    is_tone: bool,
    note_info: Option<String>,
    #[serde(serialize_with = "<[_]>::serialize")]
    nsdf: [f32; MAX_NSDF_SIZE],
    lag_count: usize,
    key_maxima_count: usize,
    selected_key_max_index: usize,
    #[serde(skip_serializing)]
    key_maxima: [KeyMaximum; MAX_KEY_MAXIMA_COUNT],
    #[serde(serialize_with = "<[_]>::serialize")]
    key_maxima_ser: [PitchReadingKeyMax; MAX_KEY_MAXIMA_COUNT],
}

impl PitchReadingInfo {
    fn new(timestamp: f32, result: &Result) -> PitchReadingInfo {
        let mut nsdf = [0.0_f32; MAX_NSDF_SIZE];
        for (i, val) in result.nsdf.iter().enumerate() {
            if i >= MAX_NSDF_SIZE {
                break;
            }
            nsdf[i] = *val;
        }
        let def = PitchReadingKeyMax {
            lag_index: 0,
            lag: 0.,
            value_at_lag_index: 0.,
            value: 0.,
        };
        let mut key_maxima_ser = [PitchReadingKeyMax::new(); MAX_KEY_MAXIMA_COUNT];
        for (i, val) in result.key_maxima.iter().enumerate() {
            key_maxima_ser[i] = PitchReadingKeyMax {
                lag_index: val.lag_index,
                value: val.value,
                value_at_lag_index: val.value_at_lag_index,
                lag: val.lag,
            }
        }

        let is_tone = result.is_tone(0.9, 0.1, 0.05);

        PitchReadingInfo {
            timestamp,
            window_size: result.window.len(),
            frequency: result.frequency,
            clarity: result.clarity,
            is_tone,
            note_number: result.note_number,
            window_rms: result.window_rms(),
            window_peak: result.window_peak(),
            selected_key_max_index: result.selected_key_max_index,
            nsdf,
            lag_count: result.nsdf.len(),
            key_maxima_count: result.key_max_count,
            key_maxima: result.key_maxima,
            key_maxima_ser,
            note_info: if is_tone { Some(note_number_to_string(result.note_number)) } else { None }
        }
    }
}

enum MPMAudioProcessorMessage {
    DetectedPitch { info: PitchReadingInfo },
}

struct MPMAudioProcessor {
    processed_sample_count: usize,
    sample_rate: f32,
    pitch_detector: Detector,
}

impl MPMAudioProcessor {
    fn new(
        sample_rate: f32
    ) -> MPMAudioProcessor {
        MPMAudioProcessor {
            processed_sample_count: 0,
            sample_rate,
            pitch_detector: Detector::new(sample_rate, 1024, 3 * 256)
        }
    }
}

impl AudioProcessor<MPMAudioProcessorMessage> for MPMAudioProcessor {
    fn process(
        &mut self,
        in_buffer: &[f32],
        out_buffer: &mut [f32],
        frame_count: usize,
        to_main_thread: &spsc::Producer<MPMAudioProcessorMessage>,
        from_main_thread: &spsc::Consumer<MPMAudioProcessorMessage>,
    ) -> bool {
        let mut sample_offset: usize = 0;
        while sample_offset < in_buffer.len() {
            match self.pitch_detector.process(&in_buffer[..], sample_offset) {
                ProcessingResult::ProcessedWindow { sample_index } => {
                    let timestamp =
                        ((self.processed_sample_count + sample_offset) as f32) / self.sample_rate;
                    let result = &self.pitch_detector.result;

                    let message = MPMAudioProcessorMessage::DetectedPitch {
                        info: PitchReadingInfo::new(timestamp, result),
                    };
                    let push_result = to_main_thread.push(message);
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
    // Create an instance of an audio processor that does pitch detection on input samples
    let sample_rate = 44100.0;
    let processor = MPMAudioProcessor::new(sample_rate);
    // Create an audio engine that provides the processor with real time input samples
    let audio_engine = AudioEngine::new(sample_rate, processor);
    println!("Started audio engine");

    // Create a websocket server for sending pitch measurements to connected clients
    let ws_server = WebsocketServer::new("127.0.0.1:9876".to_string());

    let poll_interval_ms = 30;
    let mut received_pitch_readings: Vec<PitchReadingInfo> = Vec::new();
    println!("Entering event loop, polling every {} ms", poll_interval_ms);
    println!("Open index.html in a web browser");

    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        // Get incoming websocket messages
        loop {
            match ws_server.rx_recv.try_recv() {
                Ok(value) => println!("Received websocket message on main thread {}", value),
                Err(error) => {
                    // println!("Failed to received value {}", error);
                    break;
                }
            }
        }

        // Get incoming messages from the audio thread.
        received_pitch_readings.clear();
        loop {
            match audio_engine.from_audio_thread.pop() {
                Err(reason) => {
                    // println!("Failed to pop {} on audio thread", reason);
                    break;
                }
                Ok(message) => match message {
                    MPMAudioProcessorMessage::DetectedPitch { info } => {
                        received_pitch_readings.push(info);
                    }
                },
            }
        }
        // Send the most recent pitch reading in case more than one was received
        match received_pitch_readings.last() {
            Some(info) => {
                let st = serde_json::to_string_pretty(&info).unwrap();
                let _ = ws_server.tx_send.send(st);
            }
            None => {}
        }
    }

    ws_server
        .socket_join_handle
        .join()
        .expect("Websocket thread failed");
    ws_server
        .broadcaster_join_handle
        .join()
        .expect("Broadcaster thread failed");
}
