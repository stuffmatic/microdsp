use std::sync::Mutex;

use microdsp::snov::{
    compression_function::HardKneeCompression, detector::SpectralNoveltyDetector,
};

const DEFAULT_WINDOW_SIZE: usize = 1024;

struct NoveltyDetectorWrapper {
    detector: SpectralNoveltyDetector<HardKneeCompression>,
    window_count: u64,
}

lazy_static! {
    static ref SNOV_WRAPPER: Mutex<NoveltyDetectorWrapper> = Mutex::new(NoveltyDetectorWrapper {
        detector: SpectralNoveltyDetector::new(DEFAULT_WINDOW_SIZE),
        window_count: 0
    });
}

#[no_mangle]
pub extern "C" fn snov_process(raw_buffer: *const f32, buffer_size: usize) -> bool {
    let wrapper = &mut SNOV_WRAPPER.lock().unwrap();
    let window_count_before = wrapper.window_count;
    let mut window_count = window_count_before;

    let detector = &mut wrapper.detector;

    let buffer: &[f32] = unsafe { std::slice::from_raw_parts(raw_buffer, buffer_size) };
    detector.process(buffer, |_| {
        // ignore this callback. instead, let the audio processor poll
        // the result.
        window_count += 1
    });
    wrapper.window_count = window_count;
    window_count_before < window_count
}

#[no_mangle]
pub extern "C" fn snov_get_compressed_spectrum(raw_buffer: *mut f32, max_size: usize) -> usize {
    let detector = &mut SNOV_WRAPPER.lock().unwrap().detector;
    let target_buffer: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(raw_buffer, max_size) };
    let power_spectrum = detector.novelty().power_spectrum();
    let spectrum_len = power_spectrum.len();
    target_buffer[..spectrum_len].copy_from_slice(&power_spectrum);
    spectrum_len
}

#[no_mangle]
pub extern "C" fn snov_get_spectrum_difference(raw_buffer: *mut f32, max_size: usize) -> usize {
    let detector = &mut SNOV_WRAPPER.lock().unwrap().detector;
    let target_buffer: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(raw_buffer, max_size) };
    let spectrum_difference = detector.novelty().d_power();
    let spectrum_len = spectrum_difference.len();
    target_buffer[..spectrum_len].copy_from_slice(&spectrum_difference);
    spectrum_len
}

#[no_mangle]
pub extern "C" fn snov_get_novelty() -> f32 {
    let detector = &mut SNOV_WRAPPER.lock().unwrap().detector;
    detector.novelty().novelty()
}
