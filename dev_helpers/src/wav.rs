use hound;

pub fn read_wav(path: String) -> Result<(u16, Vec<f32>), hound::Error> {
    let reader = hound::WavReader::open(path);
    match reader {
        Ok(mut reader) => {
            let samples = reader
                .samples::<i16>()
                .map(|sample| {
                    let scale = 1. / (i16::MAX as f32);
                    return (sample.unwrap() as f32) * scale;
                })
                .collect();
            return Ok((reader.spec().channels, samples));
        }
        Err(error) => return Err(error),
    }
}

pub fn write_wav(
    path: String,
    sample_rate: u32,
    channel_count: u16,
    buffer: &[f32],
) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: channel_count,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = hound::WavWriter::create(path, spec);

    match writer {
        Ok(mut writer) => {
            for sample in buffer.iter() {
                let mut clamped_sample = *sample;
                if clamped_sample < -1.0 {
                    clamped_sample = -1.
                } else if clamped_sample > 1.0 {
                    clamped_sample = 1.
                }
                let amplitude = i16::MAX as f32;
                writer
                    .write_sample((clamped_sample * amplitude) as i16)
                    .unwrap();
            }
        }
        Err(error) => return Err(error),
    }

    Ok(())
}
