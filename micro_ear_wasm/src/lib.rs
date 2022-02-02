#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use micro_ear::mpm::Detector;

lazy_static! {
    static ref DETECTOR: Mutex<Detector> = Mutex::new(Detector::new(44100., 1024, 512));
}

#[no_mangle]
pub extern "C" fn allocate_f32_array(size: usize) -> *mut f32 {
    let mut buf = Vec::<f32>::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as *mut f32
}

#[no_mangle]
pub extern "C" fn process(raw_buffer: *const f32, buffer_size: usize) {
    let mut detector = DETECTOR.lock().unwrap();

    let buffer: &[f32] = unsafe { std::slice::from_raw_parts(raw_buffer, buffer_size)};
    detector.process(buffer, |_, _| {
        // ignore this callback. instead, let the audio processor poll
        // the result.
    });
}

#[no_mangle]
pub extern "C" fn get_clarity() -> f32 {
    let detector = DETECTOR.lock().unwrap();
    detector.result().clarity
}

#[no_mangle]
pub extern "C" fn get_frequency() -> f32 {
    let detector = DETECTOR.lock().unwrap();
    detector.result().frequency
}

#[no_mangle]
pub extern "C" fn get_midi_note_number() -> f32 {
    let detector = DETECTOR.lock().unwrap();
    detector.result().midi_note_number
}

#[no_mangle]
pub extern "C" fn is_tone_with_options(clarity_threshold: f32, clarity_tolerance: f32, period_tolerance: f32) -> bool {
    let detector = DETECTOR.lock().unwrap();
    detector.result().is_tone_with_options(clarity_threshold, clarity_tolerance, period_tolerance)
}

#[no_mangle]
pub extern "C" fn is_tone() -> bool {
    let detector = DETECTOR.lock().unwrap();
    detector.result().is_tone()
}

#[no_mangle]
pub extern "C" fn set_sample_rate(sample_rate: f32) {
    let mut detector = DETECTOR.lock().unwrap();
    detector.set_sample_rate(sample_rate);
}

#[no_mangle]
pub extern "C" fn get_latest_window_timestamp() -> f64 {
    let detector = DETECTOR.lock().unwrap();
    let window_count = detector.processed_window_count() as f64;
    let sample_rate = detector.sample_rate() as f64;
    let window_distance = detector.window_distance() as f64;
    // Timestamp in seconds relative to the first window
    window_count * window_distance / sample_rate
}

#[no_mangle]
pub extern "C" fn get_latest_window_peak_level() -> f32 {
    let detector = DETECTOR.lock().unwrap();
    detector.result().window_peak()
}

#[no_mangle]
pub extern "C" fn get_latest_window_rms_level() -> f32 {
    let detector = DETECTOR.lock().unwrap();
    detector.result().window_rms()
}

#[no_mangle]
pub extern "C" fn get_processed_window_count() -> usize {
    let detector = DETECTOR.lock().unwrap();
    detector.processed_window_count()
}