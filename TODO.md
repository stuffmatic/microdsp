* https://github.com/sevagh/pitch-detection
* real-only FFT
* equal loudness filter
* slices instead of Vec
* typedef f32
* PitchDetector constructors new(sample_rate)
* PitchDetector constructors with_options(sample_rate, window_size, window_overlap, preprocessing_filter_type)
* Make PitchDetector allocate buffers for its result, allowing the result struct to be used without allocations
* Re-export types, so consumers use npm_pitch::X, Y, Z