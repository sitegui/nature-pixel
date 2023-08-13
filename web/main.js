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

        for (let pixelIndex = 0; pixelIndex < cellColorIndexes.length; pixelIndex++) {
            if (cellColorIndexes[pixelIndex] === colorIndex) {
                const x = pixelIndex % size
                const y = Math.floor(pixelIndex / size)
                context.fillRect(x, y, cellSize, cellSize)
            }
        }
    }

    requestAnimationFrame(draw)
}

draw()