//! Window processing utility.

use alloc::{vec, vec::Vec};

/// Provides fixed size windows extracted from
/// a stream of arbitrarily sized input buffers. Supports
/// downsampling and partially overlapping windows. Useful
/// for implementing algorithms operating on
/// consecutive windows of the same size.
pub struct WindowProcessor {
    downsampled_window: Vec<f32>,
    downsampling: usize,
    downsampled_hop_size: usize,
    // The write index within the downsampled window.
    write_index: usize,
    sample_counter: usize,
    should_process: bool,
}

fn validate_sizes(downsampled_size: usize, downsampled_hop_size: usize, downsampling: usize) {
    if downsampled_size == 0 {
        panic!("Downsampled size must be greater than 0")
    }
    if downsampled_hop_size == 0 {
        panic!("Downsampled hop size must be greater than 0")
    }
    if downsampling == 0 {
        panic!("Downsampling must be greater than 0")
    }
    if downsampled_hop_size > downsampled_size {
        panic!("Downsampled hop size must not be greater than downsampled size")
    }
}

impl WindowProcessor {
    /// Creates a new `WindowProcessor` instance.
    /// # Arguments
    ///
    /// * `downsampled_window_size` - The window size _after downsampling_.
    /// * `downsampled_hop_size` - The distance, _after downsampling_, between the start of collected windows. Must not be zero and not be greater than `downsampled_window_size`.
    /// * `downsampling` - The downsampling factor (1 corresponds to no downsampling)
    pub fn new(
        downsampled_window_size: usize,
        downsampled_hop_size: usize,
        downsampling: usize,
    ) -> Self {
        validate_sizes(downsampled_window_size, downsampled_hop_size, downsampling);
        WindowProcessor {
            downsampled_window: vec![0.; downsampled_window_size],
            downsampled_hop_size,
            downsampling,
            write_index: 0,
            sample_counter: 0,
            should_process: true,
        }
    }

    pub fn reset(&mut self) {
        self.write_index = 0;
        self.sample_counter = 0;
        self.should_process = true;
    }

    /// Returns the downsampling factor.
    pub fn downsampling(&self) -> usize {
        self.downsampling
    }

    /// Returns the hop size _after downsampling_.
    pub fn downsampled_hop_size(&self) -> usize {
        self.downsampled_hop_size
    }

    /// Returns the window size _after downsampling_.
    pub fn downsampled_window_size(&self) -> usize {
        self.downsampled_window.len()
    }

    /// Processes an arbitrarily sized buffer of input samples. Invokes
    /// the provided handler with each newly filled window.
    pub fn process<F>(&mut self, buffer: &[f32], mut handler: F)
    where
        F: FnMut(&[f32]),
    {
        if self.should_process {
            let downsampled_window_size = self.downsampled_window.len();
            let skip = self.sample_counter % self.downsampling;
            for input in buffer.iter().skip(skip).step_by(self.downsampling) {
                self.downsampled_window[self.write_index] = *input;
                self.write_index += 1;
                if self.write_index == downsampled_window_size {
                    handler(&self.downsampled_window);
                    self.downsampled_window
                        .rotate_left(self.downsampled_hop_size);
                    self.write_index = downsampled_window_size - self.downsampled_hop_size;
                }
            }
        }

        self.should_process = self.sample_counter < self.downsampling
            && self.sample_counter + buffer.len() >= self.downsampling;
        self.sample_counter += buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::WindowProcessor;

    #[test]
    #[should_panic]
    fn test_zero_window_size() {
        WindowProcessor::new(0, 256, 1);
    }

    #[test]
    #[should_panic]
    fn test_zero_hop_size() {
        WindowProcessor::new(256, 0, 1);
    }

    #[test]
    #[should_panic]
    fn test_too_large_hop_size() {
        WindowProcessor::new(256, 257, 1);
    }

    #[test]
    #[should_panic]
    fn test_zero_downsampling() {
        WindowProcessor::new(256, 256, 0);
    }

    #[test]
    fn test_window_processing() {
        let window_size = 15;

        // An input buffer with values 0, 1, 2, 3, 4....
        let input_buffer: Vec<f32> = (0..(5 * window_size)).map(|v| v as f32).collect();
        assert_eq!(input_buffer.len(), 5 * window_size);

        // Test various combinations of downsampling, hop size and chunk size.
        for downsampling in 1..10 {
            for hop_size in 1..=window_size {
                for chunk_size in 1..5 * window_size {
                    let mut processor = WindowProcessor::new(window_size, hop_size, downsampling);
                    let mut processed_window_count = 0;
                    let mut input_buffer_pos = 0;
                    // Feed the processor chunks of chunk_size samples
                    while input_buffer_pos < input_buffer.len() {
                        let chunk_start_idx = input_buffer_pos;
                        let current_chunk_size =
                            chunk_size.min(input_buffer.len() - chunk_start_idx);
                        let chunk_end_idx = input_buffer_pos + current_chunk_size;
                        let current_chunk_size = chunk_size.min(chunk_end_idx - chunk_start_idx);
                        let chunk = &input_buffer[chunk_start_idx..chunk_end_idx];
                        assert_eq!(chunk.len(), current_chunk_size);

                        processor.process(chunk, |window| {
                            // Verify that the first sample of the extrated window
                            // corresponds to the correct input_buffer value
                            assert_eq!(
                                window[0],
                                input_buffer[downsampling * processed_window_count * hop_size]
                            );
                            assert_eq!(window.len(), window_size);
                            processed_window_count += 1;
                        });

                        input_buffer_pos += chunk_size
                    }
                }
            }
        }
    }
}
