use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mpm_pitch::Result;

fn run_benchmark(id: &str, c: &mut Criterion, window_size: usize, lag_count: usize) {
    let mut result = Result::new(window_size, lag_count);
    c.bench_function(id, |b| {
        b.iter(|| {
            result.compute(black_box(44100.0));
        })
    });
}
fn criterion_benchmark(c: &mut Criterion) {
    run_benchmark("Window 128, lag 64", c, 128, 64);
    run_benchmark("Window 128, lag 128", c, 128, 128);

    run_benchmark("Window 256, lag 128", c, 256, 128);
    run_benchmark("Window 256, lag 256", c, 256, 256);

    run_benchmark("Window 512, lag 256", c, 512, 256);
    run_benchmark("Window 512, lag 512", c, 512, 512);

    run_benchmark("Window 1024, lag 512", c, 1024, 512);
    run_benchmark("Window 1024, lag 1024", c, 1024, 1024);

    run_benchmark("Window 2048, lag 1024", c, 2048, 1024);
    run_benchmark("Window 2048, lag 2048", c, 2048, 2048);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
