class GUI {
  isPaused = false

  // Websocket
  webSocketRetryInterval = 2.0
  webSocketUrl = "ws://127.0.0.1:9876"
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
    if (!this.isPaused) {
      this.pitchReadings = []
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