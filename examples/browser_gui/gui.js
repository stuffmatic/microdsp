class Palette {
  static plotBackground = "#f4f4f4"
  static clarity = "#FF624E"
  static pitch = "#1CA6FF"
  static nsdf = "#c0c0c0"
  static keyMaximum = "#606060"
  static rmsLevel = Palette.nsdf
}

class Canvas {
  canvasElement
  context
  constructor(canvasElementId) {
    this.canvasElement = document.getElementById(canvasElementId)
    this.context = this.canvasElement.getContext("2d")
  }

  clear(color = undefined) {
    const width = this.context.canvas.width
    const height = this.context.canvas.width
    this.context.beginPath()
    this.context.fillStyle = color ? color : Palette.plotBackground
    this.context.rect(0, 0, width, height)
    this.context.fill()
  }

  refreshSize() {
    const width = this.canvasElement.clientWidth
    const height = this.canvasElement.clientHeight
    this.canvasElement.style.width = width + "px"
    this.canvasElement.style.height = height + "px"

    const scale = window.devicePixelRatio
    this.context.canvas.width = scale * width
    this.context.canvas.height = scale * height
  }

  drawPolyline(xCoords, yCoords, color, lineWidth = 1.5) {
    const pr = window.devicePixelRatio
    this.context.lineWidth = pr * lineWidth

    this.context.beginPath()
    for (let i = 0; i < xCoords.length; i++) {
      this.context.lineTo(xCoords[i], yCoords[i])
    }
    this.context.strokeStyle = color
    this.context.stroke()
  }

  xToScreen(x, xMin, xMax) {
    const width = this.context.canvas.width
    return width * ((x - xMin) / (xMax - xMin))
  }

  yToScreen(y, yMin, yMax) {
    const height = this.context.canvas.height
    return height * (1 - (y - yMin) / (yMax - yMin))
  }
}

class PitchCanvas extends Canvas {
  render(pitchReadings, timeRange) {
    this.clear()
    if (pitchReadings.length < 2) {
      return
    }

    const timestamps = pitchReadings.map((reading) => reading.timestamp)
    const tMax = pitchReadings[pitchReadings.length - 1].timestamp

    // Draw clarity, rms and note number plots
    const curveMeta = [
      { key: "clarity", color: Palette.clarity, min: 0, max: 1 },
      { key: "window_rms", color: Palette.rmsLevel, min: 0, max: 1 }
    ]
    const noteCurveMeta = { key: "note_number", color: Palette.pitch, min: 0, max: 127 }
    curveMeta.push(noteCurveMeta)
    for (let i = 0; i < curveMeta.length; i++) {
      const meta = curveMeta[i]
      const yCoords = pitchReadings.map((r) => this.yToScreen(r[meta.key], meta.min, meta.max))
      const xCoords = timestamps.map((t) => this.xToScreen(t, tMax - timeRange, tMax))
      this.drawPolyline(xCoords, yCoords, meta.color)
    }

    // Draw thicker pitch curve when we have a discernable fundamental frequency
    const curveParts = []
    let currentCurvePart = []
    for (let i = 0; i < pitchReadings.length; i++) {
      const reading = pitchReadings[i];
      const isTone = this.hasSecondPeak(reading);
      const prevIsTone = !(i == 0 || !this.hasSecondPeak(pitchReadings[i - 1]));
      if (isTone && !prevIsTone) {
        currentCurvePart = [reading]
        if (currentCurvePart.length > 0) {
          curveParts.push(currentCurvePart)
        }
      } else if (isTone && prevIsTone) {
        currentCurvePart.push(reading);
      } else if (!isTone && prevIsTone) {

      }
    }

    for (let i = 0; i < curveParts.length; i++) {
      const curvePart = curveParts[i];
      const xCoords = curvePart.map((r) => this.xToScreen(r.timestamp, tMax - timeRange, tMax));
      const yCoords = curvePart.map((r) => this.yToScreen(r.note_number, noteCurveMeta.min, noteCurveMeta.max))
      this.drawPolyline(
        xCoords, yCoords, Palette.pitch, 12
      );
    }

  }

  hasSecondPeak(pitchReading) {
    if (pitchReading.key_maxima_count > 0) {
      const max = pitchReading.key_maxima_ser[pitchReading.selected_key_max_index];
      const maxLagIndex = max.lag_index;
      const nextLagIndex = 2 * maxLagIndex;
      const maxValue = max.value_at_lag_index;
      const nextMaxValue = pitchReading.nsdf[nextLagIndex]
      const hasSecondPeak = Math.min(maxValue, nextMaxValue) > 0.85 && Math.abs(maxValue - nextMaxValue) < 0.4;
      return hasSecondPeak
    }

    return false
  }

}

class NSDFCanvas extends Canvas {
  render(pitchReading) {
    this.clear()

    if (pitchReading === undefined) {
      return
    }

    // Compute x coordinates
    const width = this.context.canvas.width;
    const xCoords = []
    for (let i = 0; i < pitchReading.lag_count; i++) {
      xCoords.push(i * width / (pitchReading.lag_count - 1))
    }

    // Compute y coordinates
    const height = this.context.canvas.height;
    const yMin = -1
    const yMax = 1
    const yCoords = pitchReading.nsdf.map((v) => this.yToScreen(v, yMin, yMax))

    // Draw NSDF curve
    this.drawPolyline(xCoords, yCoords, Palette.nsdf)

    // Draw key maxima
    for (let i = 0; i < pitchReading.key_maxima_count; i++) {
      // Compute canvas space position of this maximum
      const x = this.xToScreen(pitchReading.key_maxima_ser[i].lag, 0, pitchReading.lag_count - 1)
      const y = this.yToScreen(pitchReading.key_maxima_ser[i].value, yMin, yMax)

      const isSelectedKeyMax = i == pitchReading.selected_key_max_index
      const radius = 0.02 * height
      this.context.beginPath()
      this.context.arc(x, y, radius, 0, 2 * Math.PI)
      this.context.fillStyle = isSelectedKeyMax ? Palette.pitch : Palette.keyMaximum
      this.context.fill()

      if (isSelectedKeyMax) {
        this.context.strokeStyle = Palette.clarity
        this.context.beginPath()
        this.context.moveTo(x, 0)
        this.context.lineTo(x, height)
        const clarityAtDoublePeriod = pitchReading.clarity_at_double_period
        if (clarityAtDoublePeriod) {
          this.context.moveTo(0, this.yToScreen(clarityAtDoublePeriod, yMin, yMax))
          this.context.lineTo(width, this.yToScreen(clarityAtDoublePeriod, yMin, yMax))
        }
        this.context.stroke()

        this.context.strokeStyle = Palette.pitch
        this.context.beginPath()
        this.context.moveTo(0, this.yToScreen(pitchReading.clarity, yMin, yMax))
        this.context.lineTo(width, this.yToScreen(pitchReading.clarity, yMin, yMax))

        this.context.stroke()
      }
    }
  }
}

class PianoCanvas extends Canvas {
  render(pitchReading) {
    this.clear()

    if (pitchReading === undefined) {
      return
    }
  }
}

class GUI {
  isPaused = false

  // Websocket
  webSocketRetryInterval = 2.0
  webSocketUrl = "ws://127.0.0.1:8080"
  webSocket = undefined
  webSocketReconnectTimer = undefined

  // HTML elements
  pitchCanvas = undefined
  nsdfCanvas = undefined
  pianoCanvas = undefined
  connectionInfoLabel = undefined

  // A list of pitch readings. Old readings get thrown
  // out as new are added
  timeRange = 2.0
  pitchReadings = []

  constructor() {
    // Get HTML elements
    this.pitchCanvas = new PitchCanvas("plot-canvas")
    this.nsdfCanvas = new NSDFCanvas("nsdf-canvas")
    this.pianoCanvas = new PianoCanvas("piano-canvas")
    this.connectionInfoLabel = document.getElementById("connection-info")

    // Hook up resize event listener
    window.addEventListener("resize", (ev) => {
      this.onResize()
    })

    this.onResize()

    // Connect to the websocket server
    this.connectToWebSocket()
  }

  setConnectionInfo(message) {
    this.connectionInfoLabel.innerText = message
  }

  connectToWebSocket() {
    if (this.webSocket === undefined) {
      this.webSocket = new WebSocket(this.webSocketUrl)
      this.webSocket.onopen = (event) => {
        window.clearInterval(this.webSocketReconnectTimer)
        this.setConnectionInfo("Connected to " + this.webSocketUrl)
        // console.log("onopen " + JSON.stringify(event));
      };
      this.webSocket.onmessage = (event) => {
        const pitchReading = JSON.parse(event.data);
        this.appendPitchReading(pitchReading)
        if (!this.isPaused) {
          this.renderCanvases()
        }
      };
      this.webSocket.onclose = (event) => {
        this.destroyWebsocket();
        // console.log("onclose " + JSON.stringify(event));
        this.webSocketReconnectTimer = window.setInterval(() => {
          this.connectToWebSocket();
        },
          this.webSocketRetryInterval * 1000
        )
        this.setConnectionInfo("Disconnected. Retrying every " + this.webSocketRetryInterval + "s.")
      };
      this.webSocket.onerror = (event) => {
        // console.log("onerror " + JSON.stringify(event));
      }
    }
  }

  destroyWebsocket() {
    if (this.webSocket !== undefined) {
      this.webSocket.onopen = undefined;
      this.webSocket.onmessage = undefined;
      this.webSocket.onclose = undefined;
      this.webSocket = undefined;
    }
  }

  appendPitchReading(newReading) {
    // Remove too old readings
    if (this.pitchReadings.length > 1) {
      const tMax = this.pitchReadings[this.pitchReadings.length - 1].timestamp
      for (let i = 0; i < this.pitchReadings.length; i++) {
        if (tMax - this.pitchReadings[i].timestamp <= this.timeRange) {
          this.pitchReadings = this.pitchReadings.slice(i)
          break
        }
      }
    }

    // Add newest reading
    this.pitchReadings.push(newReading)
  }

  togglePaused() {
    this.isPaused = !this.isPaused
  }

  onResize() {
    this.pitchCanvas.refreshSize()
    this.nsdfCanvas.refreshSize()
    this.pianoCanvas.refreshSize()

    this.renderCanvases()
  }

  renderCanvases() {
    this.pitchCanvas.render(this.pitchReadings, this.timeRange)
    const latestPitchReading = this.pitchReadings[this.pitchReadings.length - 1]
    this.nsdfCanvas.render(latestPitchReading)
    this.pianoCanvas.render(latestPitchReading)
  }
}

window.addEventListener('DOMContentLoaded', (event) => {
  const gui = new GUI()

  document.getElementById("pause-button").onclick = (e) => {
    gui.togglePaused()
  }
})