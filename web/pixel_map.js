// A wrapper around a matrix of colored pixels that continuously update itself from the server
class PixelMap {
    constructor(onload, onchange) {
        this.onload = onload
        this.onchange = onchange

        this.size = 0
        this.colors = []
        this.availableColorIndexes = []
        this.imageData = new ImageData(1, 1)
        this.versionId = null

        this.refreshEndpoint = '/api/map'
        this.refreshInterval = 1e3
        this.longPoolingTimeout = 35e3

        this.setEndpoint = '/api/cell'
        this.pendingCellChanges = []

        this._refreshLoop()
    }

    _refreshLoop() {
        const abortController = new AbortController()
        setTimeout(() => abortController.abort(), this.longPoolingTimeout)

        const lastVersionId = this.versionId ? `?last_version_id=${encodeURIComponent(this.versionId)}` : ''
        const url = `${this.refreshEndpoint}${lastVersionId}`
        fetch(url, {
            signal: abortController.signal
        }).then(response => {
            return response.json()
        }).then(response => {
            const loaded = this.versionId === null
            const changed = response.version_id !== this.versionId
            this.versionId = response.version_id
            this.size = response.size
            this.colors = response.colors
            this.availableColorIndexes = response.available_color_indexes
            this._updateImageData(response.cell_color_indexes)

            const newPendingCellChanges = []
            for (const change of this.pendingCellChanges) {
                this._setPixelData(change.xIndex, change.yIndex, change.colorIndex)

                if (change.newVersion === null || change.newVersion > this.versionId) {
                    newPendingCellChanges.push(change)
                }
            }
            this.pendingCellChanges = newPendingCellChanges

            if (loaded) {
                this.onload()
            }
            if (changed) {
                this.onchange()
            }
        }).catch(error => {
            console.error('Failed to update PixelMap', error)
        }).finally(() => {
            setTimeout(() => this._refreshLoop(), this.refreshInterval)
        })
    }

    setCellColor(xIndex, yIndex, colorIndex) {
        if (colorIndex === -1) {
            throw new Error('Invalid color')
        }

        const change = {xIndex, yIndex, colorIndex, newVersion: null}
        this.pendingCellChanges.push(change)
        this._setPixelData(xIndex, yIndex, colorIndex)

        const url = this.setEndpoint +
            '?x_index=' + encodeURIComponent(xIndex) +
            '&y_index=' + encodeURIComponent(yIndex) +
            '&color_index=' + encodeURIComponent(colorIndex)
        fetch(url, {
            method: 'POST'
        }).then(response => {
            return response.json()
        }).then(response => {
            change.newVersion = response.version_id
        }).catch(error => {
            console.error('Failed to set cell', error)
            const changeIndex = this.pendingCellChanges.indexOf(change)
            if (changeIndex !== -1) {
                this.pendingCellChanges.splice(changeIndex, 1)
                this.onchange()
            }
        })

        this.onchange()
    }

    _updateImageData(cellColorIndexes) {
        if (this.imageData.width !== this.size) {
            this.imageData = new ImageData(this.size, this.size)
            const data = this.imageData.data
            for (let i = 3; i < data.length; i += 4) {
                data[i] = 255;
            }
        }

        const colors = this.colors
        const data = this.imageData.data
        for (let i = 0, j = 0; i < cellColorIndexes.length; i++, j += 4) {
            const color = colors[cellColorIndexes[i]]
            data[j] = color[0]
            data[j + 1] = color[1]
            data[j + 2] = color[2]
        }
    }

    _setPixelData(xIndex, yIndex, colorIndex) {
        const i = yIndex * this.size + xIndex
        const color = this.colors[colorIndex]
        const data = this.imageData.data
        data[4 * i] = color[0]
        data[4 * i + 1] = color[1]
        data[4 * i + 2] = color[2]
    }
}
