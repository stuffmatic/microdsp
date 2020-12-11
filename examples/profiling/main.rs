use mpm_pitch::Result;

fn main() {
    let loop_count = 5000;
    let window_size = 1024;
    let lag_count = 512;
    let sample_rate = 44100.0;
    let mut result = Result::new(window_size, lag_count);
    println!("Computing {} pitch detection results.", loop_count);
    println!(
        "Window size {}, lag count {}, sample rate {} Hz.",
        window_size,
        lag_count,
        sample_rate
    );

    let start = std::time::Instant::now();
    for _ in 0..loop_count {
        result.compute(sample_rate)
    }
    let time_us = start.elapsed().as_micros();
    println!("Completed in {} μs ({} μs/result).", time_us, time_us / (loop_count));
    println!("");
    println!("NOTE: This example is meant for profiling.");
    println!("For performance benchmarks, run 'cargo bench'.");
}