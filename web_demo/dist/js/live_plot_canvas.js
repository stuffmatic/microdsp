class LivePlotCanvas extends CanvasBase {
  constructor(canvasElementId, channelYRanges, channelColors) {
    super(canvasElementId)
    this.channelYRanges = channelYRanges
    this.channelColors = channelColors
    this.channelCount = channelYRanges.length
    this.channelValues = []
    for (let i = 0; i < this.channelCount; i++) {
      this.channelValues.push([])
    }
    this.maxDataPointCount = 250
  }

  addDataPoint(values) {
    if (values.length != this.channelCount) {
      throw new Error("Attempting to add " + values.length + " components to plot with " + this.channelCount + " channels")
    }

    for (let channelIndex = 0; channelIndex < this.channelCount; channelIndex++) {
      const currentChannelValues = this.channelValues[channelIndex]
      currentChannelValues.push(values[channelIndex])
      if (currentChannelValues.length > this.maxDataPointCount) {
        currentChannelValues.shift(1)
      }
    }
  }

  clearDataPoints() {
    this.channelValues = []
    for (let i = 0; i < this.channelCount; i++) {
      this.channelValues.push([])
    }
    this.render()
  }

  render() {
    this.clear()
    const width = this.context.canvas.width
    const height = this.context.canvas.height
    let dx = width / (this.maxDataPointCount - 1);
    for (let channelIndex = 0; channelIndex < this.channelCount; channelIndex++) {
      const yRange = this.channelYRanges[channelIndex]
      const strokeColor = this.channelColors[channelIndex]
      const yToScreen = (y) => {
        return height - height * (y - yRange.min) / yRange.max - yRange.min
      }
      const currentChannelValues = this.channelValues[channelIndex]
      this.context.beginPath()
      let x = 0
      for (const value of currentChannelValues) {
        this.context.lineTo(x, yToScreen(value))
        x += dx
      }
      this.context.strokeStyle = strokeColor
      this.context.stroke()
    }
  }
}