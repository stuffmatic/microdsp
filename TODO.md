* high pass filter? at least mention in readme?
* fix warnings
* c bindings in c_lib
* set downsampling on the fly (and reset accumulated window)
* KeyMaximum: rename to KeyMax and make attributes private
* Clean up freq_to_midi_note
* Turn Detector into generic WindowCollector supporting hop sizes and (variable) downsampling
* micro-ear workspace with mpm-pitch and snov, dev-helpers etc as tools

## Demo/testbed

* browser based wasm demo
  * easy to run
  * dev_helpers could be removed
* ws-portaudio-browser demo
  * harder for others to build and run
  * easy debugging with breakpoints