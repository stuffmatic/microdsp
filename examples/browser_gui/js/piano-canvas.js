class IndicatorDot {
  keyNoteNumber
  alpha
  constructor() {
    this.keyNoteNumber = 0
    this.alpha = 0
  }
}

class PianoCanvas extends CanvasBase {
  numberOfOctaves = 5
  startOctave = 2 // lowest note is A{startOctave}
  indicatorDots = [
    new IndicatorDot(),
    new IndicatorDot()
  ]

  render(pitchReading) {
    this.clear("#ffffff")

    const width = this.context.canvas.width
    const height = this.context.canvas.height
    const octaveWidth = width / this.numberOfOctaves

    // Draw keys.
    const whiteKeyWidth = octaveWidth / 7
    for (let i = 0; i < this.numberOfOctaves; i++) {
      const octaveLeft = i * octaveWidth
      let whiteKeyIndex = 0
      for (let n = 0; n < 12; n++) {
        const isBlack = this.isBlackKey(n)
        const nextIsBlack = this.isBlackKey(n + 1)
        let xKey = octaveLeft + whiteKeyIndex * whiteKeyWidth
        const keyWidth = (isBlack ? 0.6 : 1) * whiteKeyWidth
        if (isBlack) {
          xKey += whiteKeyWidth - 0.5 * keyWidth
        }
        const keyHeight = (isBlack ? 0.6 : 1) * height
        this.context.beginPath()
        this.context.rect(xKey, 0, keyWidth, keyHeight)
        if (isBlack) {
          this.context.fillStyle = "#000000"
          this.context.fill()
        } else {
          this.context.strokeStyle = "#000000"
          this.context.stroke()
        }

        if (!nextIsBlack) {
          whiteKeyIndex += 1
        }
      }
    }

    if (pitchReading === undefined) {
      return
    }

    if (!pitchReading.is_tone) {
      return
    }

    const noteNumber = pitchReading.note_number

    // Draw indicator dots
    const noteNumberLow = Math.floor(noteNumber)
    const noteNumberHigh = Math.ceil(noteNumber)
    const noteNumberInterp = noteNumber - noteNumberLow

    this.indicatorDots[0].alpha = 1 - noteNumberInterp
    this.indicatorDots[0].noteNumber = noteNumberLow

    this.indicatorDots[1].alpha = noteNumberInterp
    this.indicatorDots[1].noteNumber = noteNumberHigh

    this.indicatorDots.forEach((dot) => {
      const x = this.noteNumberX(dot.noteNumber)
      const isBlack = this.isBlackKey(this.noteNumberToInOctave(dot.noteNumber))
      const y = (isBlack ? 0.43 : 0.86) * height
      this.context.globalAlpha = dot.alpha
      this.context.beginPath()
      this.context.arc(x, y, 0.4 * whiteKeyWidth, 0, 2 * Math.PI)
      this.context.fillStyle = Palette.pitch
      this.context.fill()
      this.context.globalAlpha = 1
    })
  }

  get lowestNote() {
    return 21 + this.startOctave * 12
  }

  noteNumberX(noteNumber) {
    const octaveIndex = Math.floor((noteNumber - this.lowestNote) / 12)
    const noteInOctave = (noteNumber - this.lowestNote) - 12 * octaveIndex
    const width = this.context.canvas.width
    const octaveWidth = width / this.numberOfOctaves
    const whiteKeyWidth = octaveWidth / 7
    let x = 0.5 * whiteKeyWidth
    for (let i = 0; i < noteInOctave; i++) {
      const isBlack = this.isBlackKey(i)
      const nextIsBlack = this.isBlackKey(i + 1)
      if (isBlack || nextIsBlack) {
        x += 0.5 * whiteKeyWidth
      } else {
        x += whiteKeyWidth
      }
    }

    return octaveIndex * octaveWidth + x
  }

  noteNumberToInOctave(noteNumber) {
    return Math.floor(Math.max(0, noteNumber - this.lowestNote)) % 12
  }

  isBlackKey(noteNumberInOctave) {
    return [1, 4, 6, 9, 11].indexOf(noteNumberInOctave) >= 0
  }
}