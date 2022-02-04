// Worklet config vars
const workletNodeName = "demo-wasm-processor"
const workletProcessorUrl = "/js/demo_wasm_processor.js"
const wasmUrl = "/wasm/demo_synth.wasm"

const onWorkletNodeCreated = (node) => {
  node.port.onmessage = m => {
    // console.log("mic level " + m.data.value.toFixed(3))
  }
}