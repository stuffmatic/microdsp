// Worklet config vars
const workletNodeName = "mpm-worklet-processor"
const workletProcessorUrl = "/js/mpm_worklet_processor.js"
const wasmUrl = "/wasm/microdsp_wrapper.wasm"

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
    this.context.strokeStyle = "#a0a0a0"
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
      this.drawLine(0, yScreenClarity, this.xToScreen(1, 0, 1), yScreenClarity, "rgba(255, 0, 0, " + this.measurement.clarity + ")")
      // Lag
      const xCreenLag = this.xToScreen(x, 0, n - 1)
      this.drawLine(xCreenLag, this.yToScreen(0, 0, 1), xCreenLag, this.yToScreen(1, 0, 1), "rgba(0, 0, 255, " + this.measurement.clarity + ")")
    }
  }
}

class PitchCanvas extends LivePlotCanvas {
  setMeasurement(m) {
    const dataPoint = [m.noteNumber, m.clarity, m.rmsLevel, m.peakLevel]
    this.addDataPoint(dataPoint)
  }
}

/*
Measurement:
{
  nsdf: [...],
  keyMaxPositions: [x0, y0, x1, y1, ...],
  selectedKeyMaxIndex
  frequency,
  clarity,
  noteNumber,
  rmsLevel,
  peakLevel
}
*/

pitchCanvas = null
nsdfCanvas = null

const addMeasurement = (m) => {
  nsdfCanvas.setMeasurement(m)
  pitchCanvas.setMeasurement(m)
}

const onWorkletNodeCreated = (node) => {
  node.port.onmessage = m => {
    if (m.data.type == "mpmResult") {
      addMeasurement(m.data.value)
    }
  }
  // Fake measurements
  /*setInterval((e) => {
    addMeasurement(createFakeMeasurement())
  }, 10)*/
}

const refreshCanvasSizes = () => {
  const container = document.getElementById("pitch-canvas-container")
  pitchCanvas.refreshSize(container)
  nsdfCanvas.refreshSize(container)
}

const renderCanvases = () => {
  pitchCanvas.render()
  nsdfCanvas.render()
  window.requestAnimationFrame(renderCanvases)
}

window.addEventListener('DOMContentLoaded', (event) => {
  pitchCanvas = new PitchCanvas(
    "pitch-canvas",
    [{ min: 40, max: 80}, { min: 0, max: 1}, { min: 0, max: 1}, { min: 0, max: 1}],
    ["blue", "red", "gray", "black"],
  )
  nsdfCanvas = new NSDFCanvas("nsdf-canvas")

  refreshCanvasSizes()
  renderCanvases()

  window.requestAnimationFrame(renderCanvases)


  window.addEventListener("resize", (ev) => {
    refreshCanvasSizes()
    renderCanvases()
  })
})