use alloc::{boxed::Box, vec};

///
pub struct WindowProcessor {
    downsampled_window: Box<[f32]>,
    downsampled_window_size: usize,
    downsampling: usize,
    downsampled_hop_size: usize,
    // The write index within the current sub window. A sub window
    // is a chunk of size downsampled_hop_size.
    sub_window_write_index: usize,
    // The index of the current sub window mod the number of
    // sub windows per window
    wrapped_sub_window_index: usize,
    first_read_index: usize,
    has_filled_first_window: bool,
    sample_counter: usize,
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
    if downsampled_size % downsampled_hop_size != 0 {
        // TODO: not necessary?
        panic!("Downsampled size must be divisible by downsampled hop size")
    }
}

impl WindowProcessor {
    pub fn new(
        downsampled_window_size: usize,
        downsampled_hop_size: usize,
        downsampling: usize,
    ) -> Self {
        validate_sizes(downsampled_window_size, downsampled_hop_size, downsampling);
        WindowProcessor {
            downsampled_window: vec![0.; downsampled_window_size].into_boxed_slice(),
            downsampled_window_size: downsampled_window_size,
            downsampled_hop_size,
            downsampling,
            sub_window_write_index: 0,
            wrapped_sub_window_index: 0,
            first_read_index: 0,
            has_filled_first_window: false,
            sample_counter: 0,
        }
    }

    pub fn reset(&mut self) {
        self.sub_window_write_index = 0;
        self.first_read_index = 0;
        self.wrapped_sub_window_index = 0;
        self.has_filled_first_window = false;
    }

    pub fn downsampling(&self) -> usize {
        self.downsampling
    }

    pub fn downsampled_hop_size(&self) -> usize {
        self.downsampled_hop_size
    }

    pub fn downsampled_window_size(&self) -> usize {
        self.downsampled_window_size
    }

    pub fn process<F>(&mut self, buffer: &[f32], mut handler: F)
    where
        F: FnMut(&[f32]),
    {
        let sub_windows_per_window = self.downsampled_window_size / self.downsampled_hop_size;
        for input in buffer
            .iter()
            .skip(self.first_read_index)
            .step_by(self.downsampling)
        {
            self.sample_counter += 1;
            self.downsampled_window[self.sub_window_write_index] = *input;
            self.sub_window_write_index += 1;
            if self.sub_window_write_index == self.downsampled_hop_size {
                self.wrapped_sub_window_index += 1;
                if self.wrapped_sub_window_index == sub_windows_per_window {
                    self.wrapped_sub_window_index = 0;
                    self.has_filled_first_window = true
                }

                self.sub_window_write_index = 0;

                self.downsampled_window
                    .rotate_left(self.downsampled_hop_size);
                if self.has_filled_first_window {
                    handler(&self.downsampled_window);
                }
            }
        }

        self.first_read_index = (self.first_read_index + buffer.len()) % self.downsampling
    }
}

#[cfg(test)]
mod tests {
    use super::WindowProcessor;

    #[test]
    fn test_window_processor() {
        const DOWNSAMPLED_SIZE: usize = 16;
        const BUFFER_SIZE: usize = 5 * DOWNSAMPLED_SIZE;
        const DOWNSAMPLED_HOP_SIZE: usize = 4;
        const DOWNSAMPLING: usize = 2;
        let mut processor =
            WindowProcessor::new(DOWNSAMPLED_SIZE, DOWNSAMPLED_HOP_SIZE, DOWNSAMPLING);
        let mut buffer = [0.; BUFFER_SIZE];
        for (index, value) in buffer.iter_mut().enumerate() {
            *value = index as f32;
        }

        // TODO: actual tests
        processor.process(&buffer[..1], |window| {
            /*for value in window {
                // assert_eq!(*value, processed_index as f32);
                processed_index += 1
            }*/
            let a = 0;
        });

        processor.process(&buffer[1..], |window| {
            /*for value in window {
                // assert_eq!(*value, processed_index as f32);
                processed_index += 1
            }*/
            let a = 0;
        });
    }
}
