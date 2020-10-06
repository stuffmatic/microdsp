mod audio;
mod websocket;

pub use audio::AudioEngine;
pub use audio::AudioProcessor;
pub use websocket::WebsocketServer;

pub fn note_number_to_string(note_number: f32) -> String {
  let note_names = [
      "    A",
      "A#/B♭",
      "    B",
      "    C",
      "C#/D♭",
      "    D",
      "D#/E♭",
      "    E",
      "    F",
      "F#/G♭",
      "    G",
      "G#/A♭"
  ];
  let a0_number = 21;
  let nearest_midi_note = (note_number.round() as usize).max(a0_number);
  let octave_index = (nearest_midi_note - a0_number) / 12;
  let note_in_octave = (nearest_midi_note - a0_number) - 12 * octave_index;
  let cent_offset = (100.0 * (note_number - (nearest_midi_note as f32))).round() as i32;
  let cent_sign = if cent_offset > 0 { "+" } else { "-" };
  return format!("{}-{} | {}{:02} cents", note_names[note_in_octave], octave_index, cent_sign, cent_offset.abs())
}