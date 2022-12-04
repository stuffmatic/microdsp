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
      }
    }
  }

  onWasmInstantiated(wasm) {
    this.wasm = wasm

    // Allocate buffer used for passing data to/from wasm
    const scratchBufferSize = 4096
    this.scratchBufferPointer = this.wasm.exports.allocate_f32_array(scratchBufferSize)
    this.scratchBuffer = new Float32Array(
      this.wasm.exports.memory.buffer,
      this.scratchBufferPointer,
      scratchBufferSize
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
        this.scratchBuffer.set(inputChannels[0])
        // Process input buffer
        if (this.wasm.exports.mpm_process(this.scratchBufferPointer, inputChannels[0].length)) {
          // compressed power spectrum
          const nsdfLength = this.wasm.exports.mpm_get_nsdf(this.scratchBufferPointer, this.scratchBuffer.length)
          const nsdf = new Float32Array(nsdfLength)
          nsdf.set(this.scratchBuffer.slice(0, nsdfLength))

          // key maxima
          const selectedKeyMaxIndex = this.wasm.exports.mpm_get_selected_key_max_index();
          const keyMaxCount = this.wasm.exports.mpm_get_key_maxima(this.scratchBufferPointer, this.scratchBuffer.length)
          const keyMaxPositions = new Float32Array(2 * keyMaxCount)
          keyMaxPositions.set(this.scratchBuffer.slice(0, keyMaxPositions.length))

          // misc
          const frequency = this.wasm.exports.mpm_get_frequency();
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
