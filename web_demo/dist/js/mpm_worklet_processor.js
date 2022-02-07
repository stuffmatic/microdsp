class MpmWorkletProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super(options)

    this.port.onmessage = e => {
      switch (e.data.type) {
        case "wasmData": {
          WebAssembly.instantiate(e.data.data).then(wasm => {
            this.onWasmInstantiated(wasm.instance)
          })
          break
        }
        case "toggleTone": {
          if (this.wasm) {
            this.wasm.exports.toggle_tone()
          }
          break
        }
      }
    }
  }

  onWasmInstantiated(wasm) {
    this.wasm = wasm

    // Audio worklets seem to use a fixed buffer size of 128
    const bufferSize = 128

    // Allocate sample buffers that can be passed to the wasm code
    this.inBufferPointer = this.wasm.exports.allocate_f32_array(bufferSize)
    this.inBuffer = new Float32Array(
      this.wasm.exports.memory.buffer,
      this.inBufferPointer,
      bufferSize
    )

    const maxNsdfSize = 4096;
    this.nsdfBufferPointer = this.wasm.exports.allocate_f32_array(maxNsdfSize)
    this.nsdfBuffer = new Float32Array(
      this.wasm.exports.memory.buffer,
      this.nsdfBufferPointer,
      maxNsdfSize
    )

    const maxKeyMaxCount = 128;
    this.keyMaxBufferPointer = this.wasm.exports.allocate_f32_array(maxKeyMaxCount)
    this.keyMaxBuffer = new Float32Array(
      this.wasm.exports.memory.buffer,
      this.keyMaxBufferPointer,
      maxKeyMaxCount
    )
  }

  process(inputs, outputs, parameters) {
    if (!this.wasm) {
      return true
    }
    if (inputs.length > 0) {
      const inputChannels = inputs[0]
      if (inputChannels.length > 0) {
        // Copy input samples to the wasm input buffer
        this.inBuffer.set(inputChannels[0])
        // Process input buffer
        if (this.wasm.exports.mpm_process(this.inBufferPointer, this.inBuffer.length)) {
          // nsdf
          const nsdfLength = this.wasm.exports.mpm_get_nsdf(this.nsdfBufferPointer, this.nsdfBuffer.length)
          const nsdf = new Float32Array(nsdfLength)
          const frequency = this.wasm.exports.mpm_get_frequency();
          nsdf.set(this.nsdfBuffer.slice(0, nsdfLength))

          // key maxima
          const selectedKeyMaxIndex = this.wasm.exports.mpm_get_selected_key_max_index();
          const keyMaxCount = this.wasm.exports.mpm_get_key_maxima(this.keyMaxBufferPointer, this.keyMaxBuffer.length)
          const keyMaxPositions = new Float32Array(2 * keyMaxCount)
          keyMaxPositions.set(this.keyMaxBuffer.slice(0, keyMaxPositions.length))

          // misc
          const noteNumber = this.wasm.exports.mpm_get_midi_note_number();
          const clarity = this.wasm.exports.mpm_get_clarity();
          const rmsLevel = this.wasm.exports.mpm_get_window_rms_level();
          const peakLevel = this.wasm.exports.mpm_get_window_peak_level();

          // Post result to the main thread
          this.port.postMessage({
            type: "mpmResult",
            value: {
              nsdf,
              frequency,
              keyMaxPositions,
              selectedKeyMaxIndex,
              clarity,
              noteNumber,
              rmsLevel,
              peakLevel
            }
          })
        }
      }
    }

    return true
  }
}

registerProcessor('mpm-worklet-processor', MpmWorkletProcessor)
