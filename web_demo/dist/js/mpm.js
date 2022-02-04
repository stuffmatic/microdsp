// Worklet config vars
const workletNodeName = "demo-wasm-processor"
const workletProcessorUrl = "/js/demo_wasm_processor.js"
const wasmUrl = "/wasm/demo_synth.wasm"

let fakeMeasurementPhase = 0;
let fakeMeasurementPhaseDelta = 0.01;
const createFakeMeasurement = () => {
  const nsdfLength = 512
  const nsdf = []
  for (let i = 0; i < nsdfLength; i++) {
    const amplitude = 1 - i / (nsdfLength - 1)
    nsdf.push(amplitude * Math.cos(10 * Math.PI * i / (nsdfLength - 1) + fakeMeasurementPhase))
  }

  const nKeyMax = 5
  const keyMaxPositions = []

  for (let i = 0; i < nKeyMax; i++) {
    const nsdfIndex = Math.round(0.3 * nsdfLength * i / (nKeyMax - 1))
    keyMaxPositions.push(nsdfIndex) // x
    keyMaxPositions.push(nsdf[nsdfIndex]) // y
  }
  const pitch = 1 - fakeMeasurementPhase
  const clarity = 1
  fakeMeasurementPhase = (fakeMeasurementPhase + fakeMeasurementPhaseDelta) % 1

  return {
    nsdf,
    keyMaxPositions,
    selectedKeyMaxIndex: 2,
    pitch,
    clarity
  }
}

class NSDFCanvas extends CanvasBase {
  setMeasurement(m) {
    this.measurement = m
  }

  render() {
    if (!this.measurement) {
      return
    }

    this.clear()

    // NSDF plot
    const nsdf = this.measurement.nsdf
    const n = nsdf.length
    const yMin = -1.1
    const yMax = -yMin
    this.context.beginPath()
    for (let i = 0; i < n; i++) {
      this.context.lineTo(
        this.xToScreen(i, 0, n - 1),
        this.yToScreen(nsdf[i], yMin, yMax)
      )
    }
    this.context.strokeStyle = "#e0e0e0"
    this.context.stroke()

    // Key maxima
    const keyMaxCount = this.measurement.keyMaxPositions.length / 2
    for (let i = 0; i < keyMaxCount; i++) {
      const x = this.measurement.keyMaxPositions[2 * i]
      const y = this.measurement.keyMaxPositions[2 * i + 1]
      this.fillCircle(
        this.xToScreen(x, 0, n - 1),
        this.yToScreen(y, yMin, yMax),
        "black"
      )
      if (i == this.measurement.selectedKeyMaxIndex) {
        this.fillCircle(
          this.xToScreen(x, 0, n - 1),
          this.yToScreen(y, yMin, yMax),
          "rgba(0,0,0,0.2)",
          10
        )
      }
    }

    // Clarity and lag for selected key max
    if (this.measurement.selectedKeyMaxIndex != null) {
      const x = this.measurement.keyMaxPositions[2 * this.measurement.selectedKeyMaxIndex]
      const y = this.measurement.keyMaxPositions[2 * this.measurement.selectedKeyMaxIndex + 1]
      // Clarity
      const yScreenClarity = this.yToScreen(y, yMin, yMax)
      this.drawLine(0, yScreenClarity, this.xToScreen(1, 0, 1), yScreenClarity, "red" )
      // Lag
      const xCreenLag = this.xToScreen(x, 0, n - 1)
      this.drawLine(xCreenLag, this.yToScreen(0, 0, 1), xCreenLag, this.yToScreen(1, 0, 1), "blue" )
    }
  }
}

class PitchCanvas extends CanvasBase {
  setMeasurement(m) {
    this.measurement = m
  }

  render() {
    if (!this.measurement) {
      return
    }

    this.clear()
  }
}

/*
Measurement:
{
  nsdf: [...],
  keyMaxPositions: [x0, y0, x1, y1, ...],
  selectedKeyMaxIndex
  pitch,
  clarity
}
*/
const measurements = []

pitchCanvas = null
nsdfCanvas = null

const addMeasurement = (m) => {
  nsdfCanvas.setMeasurement(m)
  nsdfCanvas.render()

  pitchCanvas.setMeasurement(m)
  pitchCanvas.render()
}

const onWorkletNodeCreated = (node) => {
  node.port.onmessage = m => {
    // console.log("mic level " + m.data.value.toFixed(3))
  }
  // Fake measurements
  setInterval((e) => {
    addMeasurement(createFakeMeasurement())
  }, 10)
}

const refreshCanvasSizes = () => {
  const container = document.getElementById("pitch-canvas-container")
  pitchCanvas.refreshSize(container)
  nsdfCanvas.refreshSize(container)
}

const renderCanvases = () => {
  pitchCanvas.render()
  nsdfCanvas.render()
}

window.addEventListener('DOMContentLoaded', (event) => {
  pitchCanvas = new PitchCanvas("pitch-canvas")
  nsdfCanvas = new NSDFCanvas("nsdf-canvas")

  refreshCanvasSizes()
  renderCanvases()

  window.addEventListener("resize", (ev) => {
    refreshCanvasSizes()
    renderCanvases()
  })


})