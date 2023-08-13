'use strict'

const app = Vue.createApp({
    data() {
        return {
            availableColors: [],
            activeColor: null,
        }
    },
    mounted() {
        this.canvas = new InteractiveCanvas(this.$refs.canvas, (x, y) => this.onClickCanvas(x, y))
        this.pixelMap = new PixelMap(() => this.onMapLoaded())
        this.draw()
    },
    methods: {
        /**
         * Redraw the canvas
         */
        draw() {
            this.canvas.prepareDraw()

            const size = this.pixelMap.size
            const cellSize = 1 / size
            const cellColorIndexes = this.pixelMap.cellColorIndexes
            const context = this.canvas.context

            for (let colorIndex = 0; colorIndex < this.pixelMap.colors.length; colorIndex++) {
                context.fillStyle = this.pixelMap.colors[colorIndex]

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

            requestAnimationFrame(() => this.draw())
        },
        /**
         * Handles user clicks on the canvas
         */
        onClickCanvas(x, y) {
            if (x >= 0 && x <= 1 && y >= 0 && y <= 1) {
                const xIndex = Math.floor(x * this.pixelMap.size)
                const yIndex = Math.floor(y * this.pixelMap.size)

                this.pixelMap.setCellColor(xIndex, yIndex, this.activeColor)
            }
        },
        /**
         * Handles when the map receives new data
         */
        onMapLoaded() {
            this.availableColors = this.pixelMap.availableColors
            this.activeColor = this.availableColors[Math.floor(Math.random() * this.availableColors.length)]
        }
    }
})

app.mount('#app')
