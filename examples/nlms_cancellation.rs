use dev_helpers::wav;
use microdsp::nlms::NlmsFilter;

fn main() {
    const FILTER_ORDER: usize = 10;
    const MU: f32 = 0.02;
    const EPS: f32 = 0.1;
    const SAMPLE_RATE: u32 = 22050;

    // Using notation from https://en.wikipedia.org/wiki/Least_mean_squares_filter

    let x_input_path = "example_data/voice_2.wav";
    let y_input_path = "example_data/voice_2_lp.wav";
    let v_input_path = "example_data/voice_1.wav";

    // x, the reference signal we want to remove from d
    let (_, x) = wav::read_wav(x_input_path.into()).unwrap();
    // y, the version of x present in d
    let (_, y) = wav::read_wav(y_input_path.into()).unwrap();
    // v, the interference, i.e a signal added to y to form d
    let (_, v) = wav::read_wav(v_input_path.into()).unwrap();
    // d, the signal to remove y from, the sum of v and y
    let d: Vec<f32> = v.iter().zip(y.iter()).map(|(v, y)| *v + *y).collect();
    println!("Created input signals");
    println!("x(n) <- {}", x_input_path);
    println!("y(n) <- {}", y_input_path);
    println!("v(n) <- {}", v_input_path);
    println!("d(n) <- v(n) + y(n)");
    println!("");

    println!("Filtering (μ={MU}, ε={EPS}, order={FILTER_ORDER})");
    println!("");

    // e, the signal formed by subtracting an estimate of y from d
    let mut e = vec![];
    let mut filter = NlmsFilter::from_options(FILTER_ORDER, MU, EPS);
    for (x, d) in x.iter().zip(d.iter()) {
        e.push(filter.update(*x, *d));
    }

    let e_output_path = "example_data/nlms_example_cancellation_e.wav";
    let x_output_path = "example_data/nlms_example_cancellation_x.wav";
    let d_output_path = "example_data/nlms_example_cancellation_d.wav";
    let _ = wav::write_wav(x_output_path.into(), SAMPLE_RATE as u32, 1, &x);
    let _ = wav::write_wav(d_output_path.into(), SAMPLE_RATE as u32, 1, &d);
    let _ = wav::write_wav(e_output_path.into(), SAMPLE_RATE as u32, 1, &e);

    println!("Wrote output signals");
    println!("x(n) -> {}", x_output_path);
    println!("d(n) -> {}", d_output_path);
    println!("e(n) -> {}", e_output_path);
}
