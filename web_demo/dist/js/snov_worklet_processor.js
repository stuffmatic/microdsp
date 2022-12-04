class SnovWorkletProcessor extends AudioWorkletProcessor {
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
        if (this.wasm.exports.snov_process(this.scratchBufferPointer, inputChannels[0].length)) {
          // power spectrum
          const spectrumLength = this.wasm.exports.snov_get_compressed_spectrum(this.scratchBufferPointer, this.scratchBuffer.length)
          const spectrum = new Float32Array(spectrumLength)
          spectrum.set(this.scratchBuffer.slice(0, spectrumLength))

          // spectrum difference
          const spectrumDifferenceLength = this.wasm.exports.snov_get_spectrum_difference(this.scratchBufferPointer, this.scratchBuffer.length);
          const spectrumDifference = new Float32Array(spectrumDifferenceLength)
          spectrumDifference.set(this.scratchBuffer.slice(0, spectrumDifferenceLength))

          // misc
          const novelty = this.wasm.exports.snov_get_novelty();

          // Post result to the main thread
          this.port.postMessage({
            type: "snovResult",
            value: {
              novelty,
              spectrum,
              spectrumDifference
            }
          })
        }
      }
    }

    return true
  }
}

registerProcessor('snov-worklet-processor', SnovWorkletProcessor)
