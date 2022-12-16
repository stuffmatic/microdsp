use micromath::F32Ext;

/// Converts a frequency in Hz to a [MIDI](https://en.wikipedia.org/wiki/MIDI) note number (with a fractional part).
pub fn freq_to_midi_note(freq: f32) -> f32 {
    12.0 * F32Ext::log2(freq) - 36.376316562295926
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_approximate_note_number() {
        // The hz to midi note conversion relies on the approximate log2
        // function of the micromath crate. This test compares this
        // approximation to std's log2 and makes sure the difference
        // is acceptable.

        // The maximum acceptable error in cents. 0.1 is 1/1000th of a semitone.
        let max_cent_error = 0.11_f32;
        for i in 1..10000 {
            let f = i as f32;
            let actual_note_number = 12.0 * (f / 440.0).log2() + 69.0;
            let approx_note_number = freq_to_midi_note(f);
            let delta_cents = 100. * (actual_note_number - approx_note_number);
            if delta_cents.abs() > max_cent_error {
                assert!(false);
            }
        }
    }
}
