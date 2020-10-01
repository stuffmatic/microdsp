class NSDFCanvas extends CanvasBase {
  render(pitchReading) {
    let clearColor = Palette.plotBackground
    if (pitchReading !== undefined && pitchReading.is_tone) {
      clearColor = Palette.plotBackgroundTone
    }
    this.clear(clearColor)

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
      this.context.fillStyle = Palette.keyMaximum
      this.context.fill()

      if (isSelectedKeyMax) {
        this.context.strokeStyle = Palette.pitch + (pitchReading.is_tone ? "" : Palette.dimmedAlpha)
        this.context.beginPath()
        this.context.moveTo(x, 0)
        this.context.lineTo(x, height)
        this.context.stroke()

        this.context.strokeStyle = Palette.clarity + (pitchReading.is_tone ? "" : Palette.dimmedAlpha)
        this.context.beginPath()

        const clarityAtDoublePeriod = pitchReading.clarity_at_double_period
        if (clarityAtDoublePeriod) {
          this.context.moveTo(0, this.yToScreen(clarityAtDoublePeriod, yMin, yMax))
          this.context.lineTo(width, this.yToScreen(clarityAtDoublePeriod, yMin, yMax))
        }

        this.context.moveTo(0, this.yToScreen(pitchReading.clarity, yMin, yMax))
        this.context.lineTo(width, this.yToScreen(pitchReading.clarity, yMin, yMax))

        this.context.stroke()
      }
    }
  }
}