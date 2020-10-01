use crossbeam_channel::unbounded;
use std::env;
use std::thread;
use std::time::Duration;
use ws::{CloseCode, Handler, Handshake, Message, Result, WebSocket};

mod audio;
use crossbeam_queue::spsc;
use mpm_pitch::KeyMaximum;
use mpm_pitch::PitchDetectionResult;
use mpm_pitch::PitchDetector;
use mpm_pitch::ProcessingResult;
use serde::Serialize;
use serde_json;

type MessageType = String;

struct WebSocketHandler {
    tx: crossbeam_channel::Sender<MessageType>,
}

trait ToneClassification {
    fn key_max_spread(&self) -> Option<f32>;
    fn clarity_at_double_period(&self) -> Option<f32>;
    fn key_max_closest_to_double_period(&self) -> Option<KeyMaximum>;
}

impl ToneClassification for PitchDetectionResult {
    // does not use interpolated lags and values. Not ideal.
    fn clarity_at_double_period(&self) -> Option<f32> {
        if self.key_max_count == 0 {
            return None
        }

        let selected_max = &self.key_maxima[self.selected_key_max_index];
        let lag_index_of_next_expected_max = 2 * selected_max.lag_index;
        if lag_index_of_next_expected_max < self.nsdf.len() {
            return Some(self.nsdf[lag_index_of_next_expected_max]);
        }

        None
    }

    //
    fn key_max_closest_to_double_period(&self) -> Option<KeyMaximum> {
        if self.key_max_count == 0 {
            return None
        }

        let selected_max = &self.key_maxima[self.selected_key_max_index];
        let lag_of_next_expected_max = 2.0 * selected_max.lag;
        let mut min_distance: f32 = 0.;
        let mut min_index: usize = 0;
        let mut found_max = false;
        let start_index = self.selected_key_max_index + 1;
        for i in start_index..self.key_max_count {
            let key_max = self.key_maxima[i];
            if key_max.lag_index == self.nsdf.len() - 1 {
                // Ignore the key max at the last lag, since it's
                // probably not a proper key maximum.
                break;
            }
            let distance = (key_max.lag - lag_of_next_expected_max).abs();
            if i == start_index {
                min_distance = distance;
                min_index = i;
            } else {
                if distance < min_distance {
                    min_distance = distance;
                    min_index = i;
                }
            }
            found_max = true;
        }

        if found_max {
            assert!(min_index > self.selected_key_max_index);
            return Some(self.key_maxima[min_index])
        }
        None
    }

    // Works well for tones with less overtones. Probably not a general solution
    fn key_max_spread(&self) -> Option<f32> {
        if self.key_max_count == 0 {
            return None;
        }

        let mut prev_lag: f32 = 0.;
        let mut min_distance = 0.0_f32;
        let mut max_distance = 0.0_f32;
        for i in 0..self.key_max_count {
            let key_max = self.key_maxima[i];
            if key_max.lag_index == self.nsdf.len() - 1 {
                // ignore last NSDF sample, since it's probably not a real maximum
                break;
            }
            let lag = key_max.lag;
            let distance = lag - prev_lag;
            if i == 0 {
                min_distance = distance;
                max_distance = distance;
            } else {
                if distance > max_distance {
                    max_distance = distance
                }
                if distance < min_distance {
                    min_distance = distance
                }
            }

            prev_lag = lag;
        }

        return Some(min_distance / max_distance);
    }
}

// https://www.jan-prochazka.eu/ws-rs/guide.html
// https://github.com/housleyjk/ws-rs/issues/131

impl Handler for WebSocketHandler {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Incoming ws message '{}'. ", msg);
        match self.tx.send(msg.to_string()) {
            Ok(_) => println!("Relayed ws message"),
            Err(e) => println!("Failed to relay ws message {}", e),
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            println!("Connection with {} now open", addr);
        }
        Ok(())
    }
}

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
    key_max_spread: Option<f32>,
    note_number: f32,
    window_rms: f32,
    window_peak: f32,
    is_tone: bool,
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
    fn new(timestamp: f32, result: &PitchDetectionResult) -> PitchReadingInfo {
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

        let is_tone = match result.key_max_closest_to_double_period() {
            Some(next_max) => {
                let max = result.key_maxima[result.selected_key_max_index];
                let delta_lag = next_max.lag - max.lag;
                let delta_value = next_max.value - max.value;
                let rel_lag_difference = delta_lag.abs() / max.lag;
                // println!("rel_lag_difference {}, delta_value {}", rel_lag_difference, delta_value);
                result.clarity > 0.8 && rel_lag_difference > 0.9 && delta_value.abs() < 0.1
            },
            None => {
                result.clarity > 0.8
            }
        };
        /*let is_tone = match result.clarity_at_double_period() {
            Some(c) => {
                result.clarity > 0.8 && (result.clarity - c).abs() < 0.1
            },
            None => {
                result.selected_key_max_index == 0 && result.clarity > 0.8
            }
        };*/
        /*let is_tone = match result.key_max_spread() {
            Some(c) => result.clarity > 0.8 && c.abs() > 0.9,
            None => false,
        };*/

        PitchReadingInfo {
            timestamp,
            window_size: result.window.len(),
            frequency: result.frequency,
            clarity: result.clarity,
            key_max_spread: result.key_max_spread(),
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
        }
    }
}

enum MPMAudioProcessorMessage {
    DetectedPitch { info: PitchReadingInfo },
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
            pitch_detector: PitchDetector::new(sample_rate, 1024, 3 * 256, false),
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

                    let message = MPMAudioProcessorMessage::DetectedPitch {
                        info: PitchReadingInfo::new(timestamp, result),
                    };
                    let push_result = self.to_main_thread.push(message);
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

    // A channel for pushing data from the main thread to the websocket for sending
    let (tx_send, rx_send) = unbounded::<MessageType>();
    // A channel for pushing incoming data from the websocket to the main thread
    let (tx_recv, rx_recv) = unbounded::<MessageType>();

    // The websocket server address
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9876".to_string());

    // Create a websocket
    let socket = WebSocket::new(move |_| WebSocketHandler {
        tx: tx_recv.clone(),
    })
    .unwrap();

    // For sending messages to all connected clients
    let broadcaster = socket.broadcaster();

    // Spawn a thread for receiving and broadcasting messages to all connected clients
    let broadcaster_join_handle = thread::spawn(move || loop {
        if let Ok(x) = rx_send.recv() {
            broadcaster
                .send(x)
                .expect("Unable to send WebSocket message.")
        } else {
            println!("Shutting down broadcaster thread.");
            break;
        }
    });

    // Spawn a thread for accepting websocket connections
    let socket_join_handle = thread::spawn(move || {
        socket.listen(addr).expect("Unable to listen on websocket");
    });

    let poll_interval_ms = 30;
    println!("Entering event loop, polling every {} ms", poll_interval_ms);
    println!("Open index.html in a web browser");

    let mut loop_count: usize = 0;
    loop {
        thread::sleep(Duration::from_millis(poll_interval_ms));

        // Get incoming websocket messages
        loop {
            match rx_recv.try_recv() {
                Ok(value) => println!("Received websocket message on main thread {}", value),
                Err(error) => {
                    // println!("Failed to received value {}", error);
                    break;
                }
            }
        }

        // Get incoming messages from the audio thread
        loop {
            match from_audio_thread.pop() {
                Err(reason) => {
                    // println!("Failed to pop {} on audio thread", reason);
                    break;
                }
                Ok(message) => match message {
                    MPMAudioProcessorMessage::DetectedPitch { info } => {
                        if loop_count % 2 == 0 {
                            let st = serde_json::to_string_pretty(&info).unwrap();
                            let _ = tx_send.send(st);
                        }
                        // println!("DetectedPitch: t={}s: {} Hz, clarity {}, RMS {} dB", timestamp, frequency, clarity, window_rms_db)
                    }
                },
            }
        }

        loop_count += 1
    }

    socket_join_handle.join().expect("Websocket thread failed");
    broadcaster_join_handle
        .join()
        .expect("Broadcaster thread failed");
}
