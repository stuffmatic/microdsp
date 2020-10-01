class PitchCanvas extends CanvasBase {
  drawMelody = true
  drawPitchAndClarity = true

  render(pitchReadings, timeRange) {
    let clearColor = Palette.plotBackground
    if (pitchReadings.length > 0) {
      if (pitchReadings[pitchReadings.length - 1].is_tone) {
        clearColor = Palette.plotBackgroundTone
      }
    }
    this.clear(clearColor)
    if (pitchReadings.length < 2) {
      return
    }

    const timestamps = pitchReadings.map((reading) => reading.timestamp)
    const tMax = pitchReadings[pitchReadings.length - 1].timestamp

    // Draw clarity, rms and note number plots
    const noteCurveMeta = { key: "note_number", color: Palette.pitch + Palette.dimmedAlpha, min: 0, max: 127 }
    if (this.drawPitchAndClarity) {
      const curveMeta = [
        { key: "clarity", color: Palette.clarity + Palette.dimmedAlpha, min: 0, max: 1 },
        { key: "window_rms", color: Palette.rmsLevel, min: 0, max: 1 }
      ]
      curveMeta.push(noteCurveMeta)
      for (let i = 0; i < curveMeta.length; i++) {
        const meta = curveMeta[i]
        const yCoords = pitchReadings.map((r) => this.yToScreen(r[meta.key], meta.min, meta.max))
        const xCoords = timestamps.map((t) => this.xToScreen(t, tMax - timeRange, tMax))
        this.drawPolyline(xCoords, yCoords, meta.color)
      }
    }

    // Draw thicker pitch curve when we have a discernable fundamental frequency
    if (this.drawPitchAndClarity) {
      const curveParts = []
      let currentCurvePart = []
      for (let i = 0; i < pitchReadings.length; i++) {
        const reading = pitchReadings[i]
        const isTone = reading.is_tone
        const prevIsTone = !(i == 0 || !pitchReadings[i - 1].is_tone)
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
          xCoords, yCoords, Palette.pitch, 8
        );
      }
    }
  }
}