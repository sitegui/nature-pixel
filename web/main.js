'use strict'

const app = Vue.createApp({
    data() {
        return {
            availableColorIndexes: [],
            activeColorIndex: null,
        }
    },
    mounted() {
        this.requiresRedraw = true
        this.canvas = new InteractiveCanvas(
            this.$refs.canvas,
            (x, y) => this.onClickCanvas(x, y),
            () => this.requiresRedraw = true,
        )
        this.pixelMap = new PixelMap(() => this.onMapLoaded(), () => this.requiresRedraw = true)

        this.pixelsBuffer = new OffscreenCanvas(1, 1)

        this.draw()
    },
    methods: {
        /**
         * The main draw loop
         */
        draw() {
            if (this.requiresRedraw && this.pixelMap.size) {
                this.requiresRedraw = false

                this.canvas.prepareDraw()
                const context = this.canvas.context

                if (this.pixelsBuffer.width !== this.pixelMap.size) {
                    this.pixelsBuffer.width = this.pixelMap.size
                    this.pixelsBuffer.height = this.pixelMap.size
                }
                this.pixelsBuffer.getContext('2d').putImageData(this.pixelMap.imageData, 0, 0)

                context.imageSmoothingEnabled = false
                context.drawImage(this.pixelsBuffer, 0, 0, this.pixelsBuffer.width, this.pixelsBuffer.height, 0, 0, 1, 1)

                const size = this.pixelMap.size
                const cellSize = 1 / size
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
            }

            requestAnimationFrame(() => this.draw())
        },
        /**
         * Handles user clicks on the canvas
         */
        onClickCanvas(x, y) {
            if (x >= 0 && x <= 1 && y >= 0 && y <= 1) {
                const xIndex = Math.floor(x * this.pixelMap.size)
                const yIndex = Math.floor(y * this.pixelMap.size)

                this.pixelMap.setCellColor(xIndex, yIndex, this.activeColorIndex)
            }
        },
        /**
         * Handles when the map receives new data
         */
        onMapLoaded() {
            this.availableColorIndexes = this.pixelMap.availableColorIndexes
            this.activeColorIndex = this.availableColorIndexes[Math.floor(Math.random() * this.availableColorIndexes.length)]
        },
        getRgbFromIndex(index) {
            const color = this.pixelMap.colors[index]
            return `rgb(${color[0]},${color[1]},${color[2]})`
        }
    }
})

app.mount('#app')
