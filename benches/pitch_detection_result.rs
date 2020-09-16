use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mpm_pitch::pitch_detection_result::PitchDetectionResult;

fn criterion_benchmark(c: &mut Criterion) {
    let window_size: usize = 1024;
    let lag_count: usize = 512;
    let mut result = PitchDetectionResult::new(window_size, lag_count);
    c.bench_function("TODO", |b| b.iter(|| {
      result.compute(black_box(44100.0)); // TODO: understand black_box
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);