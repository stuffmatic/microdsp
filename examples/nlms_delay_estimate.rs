use dev_helpers::wav;
use microdsp::nlms::NlmsFilter;

fn main() {
    // Using notation from https://en.wikipedia.org/wiki/Least_mean_squares_filter

    const SAMPLE_RATE: u32 = 22050;
    const SAMPLE_DELAY: usize = 5;
    const FILTER_ORDER: usize = 9;
    const MU: f32 = 0.1;
    const EPS: f32 = 0.01;

    #[derive(Clone, Copy)]
    struct HistogramEntry {
        count: usize,
        max_value: f32
    }

    let mut delay_estimate_histogram = [HistogramEntry { count: 0, max_value: 0. }; FILTER_ORDER];

    let signal_1_path = "example_data/voice_2.wav";
    let signal_2_path = "example_data/voice_2_reverb.wav";

    let (_, signal_1) = wav::read_wav(signal_1_path.into()).unwrap();
    let (_, signal_2) = wav::read_wav(signal_2_path.into()).unwrap();

    let delayed_signal_2: Vec<f32> = signal_2
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if i >= SAMPLE_DELAY {
                signal_2[i - SAMPLE_DELAY]
            } else {
                0.
            }
        })
        .collect();

    assert_eq!(signal_1.len(), signal_2.len());
    assert_eq!(delayed_signal_2.len(), signal_2.len());
    println!("Created input signals");
    println!("x(n) <- {}", signal_1_path);
    println!("d(n) <- {} delayed by {} samples", signal_2_path, SAMPLE_DELAY);
    println!("");

    println!("Filtering (μ={MU}, ε={EPS}, order={FILTER_ORDER})");
    println!("");
    let mut filter = NlmsFilter::<FILTER_ORDER>::from_options(MU, EPS);
    let mut e = vec![];
    for (s2, ds2) in signal_1.iter().zip(delayed_signal_2.iter()) {
        e.push(filter.update(*s2, *ds2));
        let mut max_val = 0.0;
        let mut max_idx = 0;
        for (i, h) in filter.h().iter().enumerate() {
            if *h > max_val || i == 0 {
                max_val = *h;
                max_idx = i;
            }
        }
        delay_estimate_histogram[max_idx].count += 1;
        if delay_estimate_histogram[max_idx].max_value < max_val {
            delay_estimate_histogram[max_idx].max_value = max_val
        }
    }
    let e_output_path = "example_data/nlms_example_delay_estimate_e.wav";
    let _ = wav::write_wav(e_output_path.into(), SAMPLE_RATE, 1, &e);
    println!("Wrote output signal");
    println!("e(n) -> {}", e_output_path);
    println!("");

    println!("Estimated delay    % of samples    Max peak value");
    println!("-------------------------------------------------");

    for (delay, entry) in delay_estimate_histogram.iter().enumerate() {
        print!("{}                  {:#6.3} %        {:.5}", delay, 100. * (entry.count as f32) / (signal_1.len() as f32), entry.max_value);
        if delay == SAMPLE_DELAY {
            print!("  <- Actual delay\n");
        } else {
            print!("\n");
        }
    }

    // let _ = wav::write_wav("test_signal.wav".into(), SAMPLE_RATE, 1, &signal);


}
