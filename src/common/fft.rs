use core::convert::TryInto;

pub fn real_fft_in_place(buffer: &mut [f32]) -> &mut [microfft::Complex32] {
    let fft_size = buffer.len();
    match fft_size {
        8 => microfft::real::rfft_8(buffer.try_into().unwrap()),
        16 => microfft::real::rfft_16(buffer.try_into().unwrap()),
        32 => microfft::real::rfft_16(buffer.try_into().unwrap()),
        64 => microfft::real::rfft_64(buffer.try_into().unwrap()),
        128 => microfft::real::rfft_128(buffer.try_into().unwrap()),
        256 => microfft::real::rfft_256(buffer.try_into().unwrap()),
        512 => microfft::real::rfft_512(buffer.try_into().unwrap()),
        1024 => microfft::real::rfft_1024(buffer.try_into().unwrap()),
        2048 => microfft::real::rfft_2048(buffer.try_into().unwrap()),
        4096 => microfft::real::rfft_4096(buffer.try_into().unwrap()),
        _ => panic!("Unsupported fft size {}", fft_size),
    }
}
