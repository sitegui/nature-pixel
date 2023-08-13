'use strict'

const canvas = new InteractiveCanvas(document.getElementById('canvas'))
const pixelMap = new PixelMap()

function draw() {
    canvas.prepareDraw()

    const size = pixelMap.size
    const cellSize = 1 / size
    const cellColorIndexes = pixelMap.cellColorIndexes
    const context = canvas.context

    for (let colorIndex = 0; colorIndex < pixelMap.colors.length; colorIndex++) {
        context.fillStyle = pixelMap.colors[colorIndex]

        for (let yIndex = 0; yIndex < size; yIndex++) {
            const pixelIndexOffset = yIndex * size
            for (let xIndex = 0; xIndex < size; xIndex++) {
                if (cellColorIndexes[pixelIndexOffset + xIndex] === colorIndex) {
                    context.fillRect(xIndex * cellSize, yIndex * cellSize, cellSize, cellSize)
                }
            }
        }
    }

    context.strokeStyle = 'black'
    context.lineWidth = cellSize / 100
    context.beginPath()
    for (let i = 0; i <= size; i++) {
        context.moveTo(i * cellSize, 0)
        context.lineTo(i * cellSize, 1)

        context.moveTo(0, i * cellSize)
        context.lineTo(1, i * cellSize)
    }
    context.stroke()

    context.strokeStyle = 'gray'
    context.lineWidth = cellSize / 10
    context.strokeRect(0, 0, 1, 1)

    requestAnimationFrame(draw)
}

draw()

canvas.element.addEventListener('click', event => {
    const position = canvas.convertToUnit(event.clientX, event.clientY)

    if (position.x >= 0 && position.x <= 1 && position.y >= 0 && position.y <= 1) {
        const xIndex = Math.floor(position.x * pixelMap.size)
        const yIndex = Math.floor(position.y * pixelMap.size)

        pixelMap.setCellColor(xIndex, yIndex, 'limegreen')
    }
})