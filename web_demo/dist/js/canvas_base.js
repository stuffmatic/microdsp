class CanvasBase {
  constructor(canvasElementId) {
    this.canvasElement = document.getElementById(canvasElementId)
    this.context = this.canvasElement.getContext("2d")
  }

  clear() {
    const width = this.context.canvas.width
    const height = this.context.canvas.height
    /*this.context.beginPath()
    this.context.fillStyle = "rgba(0,0,0,0)"
    this.context.rect(0, 0, width, height)
    this.context.fill()*/
    this.context.clearRect(0, 0, width, height)
  }

  refreshSize(container) {
    const width = container.clientWidth
    const height = container.clientHeight

    const scale = window.devicePixelRatio
    this.context.canvas.width = scale * width
    this.context.canvas.height = scale * height

    const pr = window.devicePixelRatio
    const lineWidth = 1.5
    this.context.lineWidth = pr * lineWidth
  }

  drawPolyline(xCoords, yCoords, color) {
    this.context.beginPath()
    for (let i = 0; i < xCoords.length; i++) {
      this.context.lineTo(xCoords[i], yCoords[i])
    }
    this.context.strokeStyle = color
    this.context.stroke()
  }

  drawLine(xStartCoord, yStartCoord, xEndCoord, yEndCoord, color) {
    this.drawPolyline([xStartCoord, xEndCoord], [yStartCoord, yEndCoord], color)
  }

  xToScreen(x, xMin, xMax) {
    const width = this.context.canvas.width
    return width * ((x - xMin) / (xMax - xMin))
  }

  yToScreen(y, yMin, yMax) {
    const height = this.context.canvas.height
    return height * (1 - (y - yMin) / (yMax - yMin))
  }

  fillCircle(x, y, color, radius = 3) {
    this.context.beginPath()
    this.context.arc(x, y, radius, 0, 2 * Math.PI)
    this.context.fillStyle = color
    this.context.fill()
  }

  strokeCircle(x, y, color, radius = 3) {
    this.context.beginPath()
    this.context.arc(x, y, radius, 0, 2 * Math.PI)
    this.context.fillStyle = color
    this.context.stroke()
  }

  renderTest() {
    this.clear("orange")
    this.drawLine(
      this.xToScreen(0, 0, 1),
      this.yToScreen(0, 0, 1),
      this.xToScreen(1, 0, 1),
      this.yToScreen(1, 0, 1),
      "white"
    )
  }
}