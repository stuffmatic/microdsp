use criterion::{black_box, criterion_group, criterion_main, Criterion};
use micro_mpm::Detector;
use micro_mpm::Result;

fn run_result_benchmark(id: &str, c: &mut Criterion, window_size: usize, lag_count: usize) {
    let mut result = Result::new(window_size, lag_count);
    c.bench_function(id, |b| {
        b.iter(|| {
            result.compute(black_box(44100.0));
        })
    });
}
fn result_benchmarks(c: &mut Criterion) {
    run_result_benchmark("Window 128, lag 64", c, 128, 64);
    run_result_benchmark("Window 128, lag 128", c, 128, 128);

    run_result_benchmark("Window 256, lag 128", c, 256, 128);
    run_result_benchmark("Window 256, lag 256", c, 256, 256);

    run_result_benchmark("Window 512, lag 256", c, 512, 256);
    run_result_benchmark("Window 512, lag 512", c, 512, 512);

    run_result_benchmark("Window 1024, lag 512", c, 1024, 512);
    run_result_benchmark("Window 1024, lag 1024", c, 1024, 1024);

    run_result_benchmark("Window 2048, lag 1024", c, 2048, 1024);
    run_result_benchmark("Window 2048, lag 2048", c, 2048, 2048);
}

fn run_detector_benchmark(id: &str, c: &mut Criterion, window_size: usize, downsampling_factor: usize) {
    let mut detector = Detector::from_options(
        44100.,
        window_size,
        window_size,
        window_size / 2,
        downsampling_factor,
    );
    let input_buffer = vec![0.0; window_size];

    c.bench_function(id, |b| {
        b.iter(|| {
            detector.process(black_box(&input_buffer[..]), |_, _| {

            })
        })
    });
}
fn detector_benchmarks(c: &mut Criterion) {
    run_detector_benchmark("Window 2048, downsampling 1", c, 2048, 1);
    run_detector_benchmark("Window 2048, downsampling 2", c, 2048, 2);
    run_detector_benchmark("Window 2048, downsampling 4", c, 2048, 4);
    run_detector_benchmark("Window 2048, downsampling 8", c, 2048, 8);
    run_detector_benchmark("Window 2048, downsampling 16", c, 2048, 16);
}

criterion_group!(benches, detector_benchmarks, result_benchmarks);
criterion_main!(benches);

