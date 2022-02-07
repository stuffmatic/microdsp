#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use microear::{mpm::PitchDetector};

struct PitchDetectorWrapper {
    detector: PitchDetector,
    window_count: u64
}

lazy_static! {
    static ref WRAPPER: Mutex<PitchDetectorWrapper> = Mutex::new(
        PitchDetectorWrapper {
            detector: PitchDetector::new(44100., 1024, 512),
            window_count: 0
        }
    );
}

#[no_mangle]
pub extern "C" fn allocate_f32_array(size: usize) -> *mut f32 {
    let mut buf = Vec::<f32>::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as *mut f32
}

#[no_mangle]
pub extern "C" fn mpm_process(raw_buffer: *const f32, buffer_size: usize) -> bool {
    let wrapper = &mut WRAPPER.lock().unwrap();
    let window_count_before = wrapper.window_count;
    let mut window_count = window_count_before;

    let detector = &mut wrapper.detector;

    let buffer: &[f32] = unsafe { std::slice::from_raw_parts(raw_buffer, buffer_size)};
    detector.process(buffer, |_| {
        // ignore this callback. instead, let the audio processor poll
        // the result.
        window_count += 1
    });
    wrapper.window_count = window_count;
    window_count_before < window_count
}

#[no_mangle]
pub extern "C" fn mpm_get_nsdf(raw_buffer: *mut f32, max_size: usize) -> usize {
    let detector = &WRAPPER.lock().unwrap().detector;
    let nsdf_buffer: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(raw_buffer, max_size)};
    let nsdf_len = detector.result().nsdf.len();
    nsdf_buffer[..nsdf_len].copy_from_slice(&detector.result().nsdf);
    nsdf_len
}

#[no_mangle]
pub extern "C" fn mpm_get_key_maxima(raw_buffer: *mut f32, max_count: usize) -> usize {
    let detector = &WRAPPER.lock().unwrap().detector;
    let key_max_xy_pairs: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(raw_buffer, 2 * max_count)};
    let key_max_count = detector.result().key_max_count;
    for i in 0..key_max_count {
        key_max_xy_pairs[2 * i + 0] = detector.result().key_maxima[i].lag;
        key_max_xy_pairs[2 * i + 1] = detector.result().key_maxima[i].value;
    }

    key_max_count
}

#[no_mangle]
pub extern "C" fn mpm_get_selected_key_max_index() -> usize {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().selected_key_max_index
}

#[no_mangle]
pub extern "C" fn mpm_get_clarity() -> f32 {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().clarity
}

#[no_mangle]
pub extern "C" fn mpm_get_frequency() -> f32 {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().frequency
}

#[no_mangle]
pub extern "C" fn mpm_get_midi_note_number() -> f32 {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().midi_note_number
}

#[no_mangle]
pub extern "C" fn mpm_is_tone_with_options(clarity_threshold: f32, clarity_tolerance: f32, period_tolerance: f32) -> bool {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().is_tone_with_options(clarity_threshold, clarity_tolerance, period_tolerance)
}

#[no_mangle]
pub extern "C" fn mpm_is_tone() -> bool {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().is_tone()
}

#[no_mangle]
pub extern "C" fn mpm_set_sample_rate(sample_rate: f32) {
    let detector = &mut WRAPPER.lock().unwrap().detector;
    detector.set_sample_rate(sample_rate);
}

#[no_mangle]
pub extern "C" fn mpm_get_window_peak_level() -> f32 {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().window_peak()
}

#[no_mangle]
pub extern "C" fn mpm_get_window_rms_level() -> f32 {
    let detector = &WRAPPER.lock().unwrap().detector;
    detector.result().window_rms()
}