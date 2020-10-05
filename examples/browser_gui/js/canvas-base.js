class CanvasBase {
  constructor(canvasElementId) {
    this.canvasElement = document.getElementById(canvasElementId)
    this.context = this.canvasElement.getContext("2d")
  }

  clear(color = undefined) {
    const width = this.context.canvas.width
    const height = this.context.canvas.height
    this.context.beginPath()
    this.context.fillStyle = color ? color : Palette.plotBackground
    this.context.rect(0, 0, width, height)
    this.context.fill()
  }

  refreshSize() {
    const width = this.canvasElement.clientWidth
    const height = this.canvasElement.clientHeight

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