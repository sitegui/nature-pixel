'use strict'

// A wrapper around canvas that allows the user to pan and zoom using the mouse and the finger.
// Content drawn in the unit square from (0, 0) to (1, 1) will initially fit the whole canvas
class InteractiveCanvas {
    constructor(canvas) {
        this.canvas = canvas

        const boundingRect = this.canvas.getBoundingClientRect()
        this.canvas.width = boundingRect.width
        this.canvas.height = boundingRect.height
        this.scale = Math.min(this.canvas.width, this.canvas.height)
        this.x = (this.canvas.width - this.scale) / 2
        this.y = (this.canvas.height - this.scale) / 2

        this.context = this.canvas.getContext('2d')

        this.dragAnchor1 = null
        this.dragAnchor2 = null
        this.initialPinchDistance = 0
        this.initialPinchScale = 0

        this.maxScale = 5 * this.scale
        this.minScale = 0.1 * this.scale
        this.scrollSensitivity = 0.0005

        this.canvas.addEventListener('mousedown', event => this._onMouseDown(event))
        this.canvas.addEventListener('mousemove', event => this._onMouseMove(event))
        this.canvas.addEventListener('mouseup', () => this._onMouseUp())
        this.canvas.addEventListener('wheel', event => this._onWheel(event))

        this.canvas.addEventListener('touchstart', event => this._onTouchStart(event))
        this.canvas.addEventListener('touchmove', event => this._onTouchMove(event))
        this.canvas.addEventListener('touchend', () => this._onTouchEnd())
    }

    // Prepare the context transform matrix so that drawing in the unit square draws into the desired region
    prepareDraw() {
        this.context.resetTransform()
        this.context.clearRect(0, 0, this.canvas.width, this.canvas.height)
        this.context.translate(this.x, this.y)
        this.context.scale(this.scale, this.scale)
    }

    _getDragAnchor(id) {
        if (this.dragAnchor1 && this.dragAnchor1.id === id) {
            return this.dragAnchor1
        }
        if (this.dragAnchor2 && this.dragAnchor2.id === id) {
            return this.dragAnchor2
        }
        return null
    }

    _onMouseDown(event) {
        this.dragAnchor1 = DragAnchor.fromMouse(this, event)
        this.dragAnchor2 = null
    }

    _onMouseMove(event) {
        const anchor = this._getDragAnchor(null)
        if (anchor) {
            this._padForAnchor(anchor, DragAnchor.fromMouse(this, event))
        }
    }

    _padForAnchor(anchor, newAnchor) {
        this.x = newAnchor.elementX - anchor.x * this.scale
        this.y = newAnchor.elementY - anchor.y * this.scale
    }

    _onMouseUp() {
        this.dragAnchor1 = null
        this.dragAnchor2 = null
    }

    _onWheel(event) {
        const anchor = DragAnchor.fromMouse(this, event)

        const amount = 1 + event.deltaY * this.scrollSensitivity
        const newScale = Math.max(this.minScale, Math.min(this.maxScale, this.scale * amount))
        this.x -= anchor.x * (newScale - this.scale)
        this.y -= anchor.y * (newScale - this.scale)
        this.scale = newScale
    }

    _onTouchStart(event) {
        if (event.touches.length === 1) {
            this.dragAnchor1 = DragAnchor.fromTouch(this, event.touches[0])
            this.dragAnchor2 = null
        } else if (event.touches.length === 2) {
            this._startPinch(
                DragAnchor.fromTouch(this, event.touches[0]),
                DragAnchor.fromTouch(this, event.touches[1]),
            )
        } else {
            this.dragAnchor1 = null
            this.dragAnchor2 = null
        }
    }

    _onTouchMove(event) {
        if (event.touches.length === 1) {
            const anchor = this._getDragAnchor(event.touches[0].identifier)
            if (anchor) {
                const newAnchor = DragAnchor.fromTouch(this, event.touches[0])
                this._padForAnchor(anchor, newAnchor)
            }
        } else if (event.touches.length === 2) {
            const anchor1 = this._getDragAnchor(event.touches[0].identifier)
            const newAnchor1 = DragAnchor.fromTouch(this, event.touches[0])
            const anchor2 = this._getDragAnchor(event.touches[1].identifier)
            const newAnchor2 = DragAnchor.fromTouch(this, event.touches[1])

            if (!anchor1 && !anchor2) {
                this._startPinch(newAnchor1, newAnchor2)
            } else if (anchor1 && !anchor2) {
                this._padForAnchor(anchor1, newAnchor1)
                this._startPinch(anchor1, newAnchor2)
            } else if (!anchor1 && anchor2) {
                this._padForAnchor(anchor2, newAnchor2)
                this._startPinch(newAnchor1, anchor2)
            } else {
                this._padForAnchor(DragAnchor.fromMiddle(anchor1, anchor2), DragAnchor.fromMiddle(newAnchor1, newAnchor2))
                const pinchRatio = newAnchor1.elementDistanceTo(newAnchor2) / this.initialPinchDistance
                this.scale = Math.max(this.minScale, Math.min(this.maxScale, this.initialPinchScale * pinchRatio))
            }
        }
    }

    _startPinch(anchor1, anchor2) {
        this.dragAnchor1 = anchor1
        this.dragAnchor2 = anchor2
        this.initialPinchScale = this.scale
        this.initialPinchDistance = anchor1.elementDistanceTo(anchor2)
    }

    _onTouchEnd() {
        this.dragAnchor1 = null
        this.dragAnchor2 = null
    }
}

class DragAnchor {


    constructor(id, elementX, elementY, x, y) {
        this.id = id
        this.elementX = elementX
        this.elementY = elementY
        this.x = x
        this.y = y
    }

    static fromMouse(canvas, event) {
        return DragAnchor._fromCanvas(canvas, null, event.clientX, event.clientY)
    }

    static fromTouch(canvas, touch) {
        return DragAnchor._fromCanvas(canvas, touch.identifier, touch.clientX, touch.clientY)
    }

    static _fromCanvas(canvas, id, clientX, clientY) {
        const boundingRect = canvas.canvas.getBoundingClientRect()
        const elementX = clientX - boundingRect.x
        const elementY = clientY - boundingRect.y
        const x = (elementX - canvas.x) / canvas.scale
        const y = (elementY - canvas.y) / canvas.scale
        return new DragAnchor(id, elementX, elementY, x, y)
    }

    static fromMiddle(a, b) {
        return new DragAnchor(
            null,
            (a.elementX + b.elementX) / 2,
            (a.elementY + b.elementY) / 2,
            (a.x + b.x) / 2,
            (a.y + b.y) / 2,
        )
    }

    elementDistanceTo(another) {
        const dx = this.elementX - another.elementX
        const dy = this.elementY - another.elementY
        return Math.hypot(dx, dy)
    }
}

const gui = new InteractiveCanvas(document.getElementById('canvas'))

function drawGui() {
    gui.prepareDraw()

    gui.context.fillStyle = 'yellow'
    gui.context.fillRect(0, 0, 0.5, 0.5)
    gui.context.fillStyle = 'green'
    gui.context.fillRect(0.5, 0, 0.5, 0.5)
    gui.context.fillStyle = 'blue'
    gui.context.fillRect(0, 0.5, 0.5, 0.5)
    gui.context.fillStyle = 'black'
    gui.context.fillRect(0.5, 0.5, 0.5, 0.5)

    requestAnimationFrame(drawGui)
}

drawGui()