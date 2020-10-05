class GUI {
  constructor() {
    this.isPaused = false

    this.timeRange = 2.0
    this.frequencyPlotPoints = []
    this.latestPitchReading = undefined

    // Get HTML elements
    this.pitchCanvas = new PitchCanvas("plot-canvas")
    this.nsdfCanvas = new NSDFCanvas("nsdf-canvas")
    this.pianoCanvas = new PianoCanvas("piano-canvas")
    this.connectionInfoLabel = document.getElementById("connection-info")

    // Websocket
    this.webSocketRetryInterval = 2.0
    this.webSocketUrl = "ws://127.0.0.1:9876"
    this.webSocket = undefined
    this.webSocketReconnectTimer = undefined

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
        if (!this.isPaused) {
          const pitchReading = JSON.parse(event.data);
          // console.log(pitchReading.note_number)
          this.appendPitchReading(pitchReading)
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
    this.latestPitchReading = newReading

    // Remove too old readings
    if (this.frequencyPlotPoints.length > 1) {
      const tMax = this.frequencyPlotPoints[this.frequencyPlotPoints.length - 1].timestamp
      for (let i = 0; i < this.frequencyPlotPoints.length; i++) {
        if (tMax - this.frequencyPlotPoints[i].timestamp <= this.timeRange) {
          this.frequencyPlotPoints = this.frequencyPlotPoints.slice(i)
          break
        }
      }
    }

    // Add newest reading
    this.frequencyPlotPoints.push({
      timestamp: newReading.timestamp,
      clarity: newReading.clarity,
      noteNumber: newReading.note_number,
      rmsLevel: newReading.window_rms,
      isTone: newReading.is_tone
    })
  }

  togglePaused() {
    this.isPaused = !this.isPaused
    if (!this.isPaused) {
      this.latestPitchReading = undefined
      this.pitchReadings = []
      this.frequencyPlotPoints = []
    }
    this.renderCanvases()
  }

  onResize() {
    this.pitchCanvas.refreshSize()
    this.nsdfCanvas.refreshSize()
    this.pianoCanvas.refreshSize()

    this.renderCanvases()
  }

  renderCanvases() {
    this.pitchCanvas.render(this.frequencyPlotPoints, this.timeRange)
    this.nsdfCanvas.render(this.latestPitchReading)
    this.pianoCanvas.render(this.latestPitchReading)
  }
}

window.addEventListener('DOMContentLoaded', (event) => {
  const gui = new GUI()

  document.getElementById("pause-button").onclick = (e) => {
    gui.togglePaused()
  }
})