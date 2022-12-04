use std::sync::Mutex;

use microdsp::mpm::PitchDetector;

const DEFAULT_WINDOW_SIZE: usize = 1024;
const DEFAULT_HOP_SIZE: usize = 512;
const DEFAULT_LAG_COUNT: usize = 512;
const DEFAULT_SAMPLE_RATE: f32 = 44100.;

struct PitchDetectorWrapper {
    detector: PitchDetector,
    window_count: u64,
}

lazy_static! {
    static ref MPM_WRAPPER: Mutex<PitchDetectorWrapper> = Mutex::new(PitchDetectorWrapper {
        detector: PitchDetector::from_options(DEFAULT_SAMPLE_RATE, DEFAULT_WINDOW_SIZE, DEFAULT_HOP_SIZE, DEFAULT_LAG_COUNT, 1),
        window_count: 0
    });
}

#[no_mangle]
pub extern "C" fn mpm_process(raw_buffer: *const f32, buffer_size: usize) -> bool {
    let wrapper = &mut MPM_WRAPPER.lock().unwrap();

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
pub extern "C" fn mpm_get_nsdf(raw_buffer: *mut f32, max_size: usize) -> usize {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    let nsdf_buffer: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(raw_buffer, max_size) };
    let nsdf_len = detector.result().nsdf.len();
    nsdf_buffer[..nsdf_len].copy_from_slice(&detector.result().nsdf);
    nsdf_len
}

#[no_mangle]
pub extern "C" fn mpm_get_key_maxima(raw_buffer: *mut f32, max_count: usize) -> usize {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    let key_max_xy_pairs: &mut [f32] =
        unsafe { std::slice::from_raw_parts_mut(raw_buffer, 2 * max_count) };
    let key_max_count = detector.result().key_max_count;
    for i in 0..key_max_count {
        key_max_xy_pairs[2 * i + 0] = detector.result().key_maxima[i].lag;
        key_max_xy_pairs[2 * i + 1] = detector.result().key_maxima[i].value;
    }

    key_max_count
}

#[no_mangle]
pub extern "C" fn mpm_set_downsampling(downsampling: usize) {
    let wrapper = &mut MPM_WRAPPER.lock().unwrap();
    let downsampled_window_size = DEFAULT_WINDOW_SIZE / downsampling;
    wrapper.detector = PitchDetector::from_options(
        DEFAULT_SAMPLE_RATE,
        downsampled_window_size,
        downsampled_window_size / 2,
        downsampled_window_size / 2,
        downsampling
    );
}

#[no_mangle]
pub extern "C" fn mpm_get_selected_key_max_index() -> usize {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().selected_key_max_index
}

#[no_mangle]
pub extern "C" fn mpm_get_clarity() -> f32 {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().clarity
}

#[no_mangle]
pub extern "C" fn mpm_get_frequency() -> f32 {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().frequency
}

#[no_mangle]
pub extern "C" fn mpm_get_midi_note_number() -> f32 {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().midi_note_number
}

#[no_mangle]
pub extern "C" fn mpm_is_tone_with_options(
    clarity_threshold: f32,
    clarity_tolerance: f32,
    period_tolerance: f32,
) -> bool {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector
        .result()
        .is_tone_with_options(clarity_threshold, clarity_tolerance, period_tolerance)
}

#[no_mangle]
pub extern "C" fn mpm_is_tone() -> bool {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().is_tone()
}

#[no_mangle]
pub extern "C" fn mpm_set_sample_rate(sample_rate: f32) {
    let detector = &mut MPM_WRAPPER.lock().unwrap().detector;
    detector.set_sample_rate(sample_rate);
}

#[no_mangle]
pub extern "C" fn mpm_get_window_peak_level() -> f32 {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().window_peak()
}

#[no_mangle]
pub extern "C" fn mpm_get_window_rms_level() -> f32 {
    let detector = &MPM_WRAPPER.lock().unwrap().detector;
    detector.result().window_rms()
}
