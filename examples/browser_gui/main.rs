use crossbeam_channel::unbounded;
use std::env;
use std::thread;
use std::time::Duration;
use ws::{CloseCode, Handler, Handshake, Message, Result, WebSocket};

mod audio;
use crossbeam_queue::spsc;
use mpm_pitch::pitch_detector::PitchDetector;
use mpm_pitch::pitch_detector::ProcessingResult;

type MessageType = String;

struct WebSocketHandler {
    tx: crossbeam_channel::Sender<MessageType>,
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

    // A channel for pushing data from the main thread to the websocket for sending
    let (tx_send, rx_send) = unbounded::<MessageType>();
    // A channel for pushing incoming data from the websocket to the main thread
    let (tx_recv, rx_recv) = unbounded::<MessageType>();

    // The websocket server address
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

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
                        let _ = tx_send.send(
                            format!(
                                "{{\"t\": {}, \"f\":{}, \"c\": {}, \"n\": {}, \"l\": {}, \"lp\": {}}}",
                                timestamp,
                                frequency,
                                clarity,
                                note_number,
                                window_rms,
                                window_peak
                            )
                        );
                        // println!("DetectedPitch: t={}s: {} Hz, clarity {}, RMS {} dB", timestamp, frequency, clarity, window_rms_db)
                    }
                },
            }
        }
    }

    socket_join_handle.join().expect("Websocket thread failed");
    broadcaster_join_handle.join().expect("Broadcaster thread failed");
}
