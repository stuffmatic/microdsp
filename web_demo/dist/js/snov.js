// Worklet config vars
const workletNodeName = "snov-worklet-processor"
const workletProcessorUrl = "/js/snov_worklet_processor.js"
const wasmUrl = "/wasm/microdsp_wrapper.wasm"

let noveltyCanvas = null

const onWorkletNodeCreated = (node) => {
  node.port.onmessage = m => {
    if (m.data.type == "snovResult") {
      const value = m.data.value.novelty

      noveltyCanvas.addDataPoint([value])
    }
  }
  // Fake measurements
  /*setInterval((e) => {
    addMeasurement(createFakeMeasurement())
  }, 10)*/
}

const refreshCanvasSizes = () => {
  const container = document.getElementById("novelty-canvas-container")
  noveltyCanvas.refreshSize(container)
}

const renderCanvases = () => {
  noveltyCanvas.render()
}

window.addEventListener('DOMContentLoaded', (event) => {
  noveltyCanvas = new LivePlotCanvas(
    "novelty-canvas",
    [{ min: -1, max: 10}],
    ["black"],
  )

  refreshCanvasSizes()
  renderCanvases()

  window.addEventListener("resize", (ev) => {
    refreshCanvasSizes()
    renderCanvases()
  })
})