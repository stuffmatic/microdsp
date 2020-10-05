class PitchCanvas extends CanvasBase {
  constructor(canvasElementId) {
    super(canvasElementId)
    this.drawMelody = true
    this.drawPitchAndClarity = true
  }

  render(plotPoints, timeRange) {
    let clearColor = Palette.plotBackground
    if (plotPoints !== undefined && plotPoints.length > 0) {
      if (plotPoints[plotPoints.length - 1].isTone) {
        clearColor = Palette.plotBackgroundTone
      }
    }
    this.clear(clearColor)
    if (plotPoints === undefined || plotPoints.length < 2) {
      return
    }

    const timestamps = plotPoints.map((reading) => reading.timestamp)
    const tMax = plotPoints[plotPoints.length - 1].timestamp

    // Draw clarity, rms and note number plots
    const noteCurveMeta = { key: "noteNumber", color: Palette.pitch + Palette.dimmedAlpha, min: 21, max: 127 }
    if (this.drawPitchAndClarity) {
      const curveMeta = [
        { key: "clarity", color: Palette.clarity + Palette.dimmedAlpha, min: 0, max: 1 },
        { key: "rmsLevel", color: Palette.rmsLevel, min: 0, max: 1 }
      ]
      curveMeta.push(noteCurveMeta)
      for (let i = 0; i < curveMeta.length; i++) {
        const meta = curveMeta[i]
        const yCoords = plotPoints.map((r) => this.yToScreen(r[meta.key], meta.min, meta.max))

        const xCoords = timestamps.map((t) => {
          return this.xToScreen(t, tMax - timeRange, tMax)
        })
        this.drawPolyline(xCoords, yCoords, meta.color)
      }
    }

    // Draw thicker pitch curve when we have a discernable fundamental frequency
    if (this.drawPitchAndClarity) {
      const curveParts = []
      let currentCurvePart = []
      for (let i = 0; i < plotPoints.length; i++) {
        const reading = plotPoints[i]
        const isTone = reading.isTone
        const prevIsTone = !(i == 0 || !plotPoints[i - 1].isTone)
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
        const yCoords = curvePart.map((r) => this.yToScreen(r.noteNumber, noteCurveMeta.min, noteCurveMeta.max))

        this.drawPolyline(
          xCoords, yCoords, Palette.pitch, 8
        );
      }
    }
  }
}