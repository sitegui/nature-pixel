// A wrapper around a matrix of colored pixels that continuously update itself from the server
class PixelMap {
    constructor() {
        this.size = 100
        this.colors = ['white']
        this.cellColorIndexes = new Array(this.size * this.size).fill(0)
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
            this.versionId = response.version_id
            this.size = response.size
            this.colors = response.colors
            this.cellColorIndexes = response.cell_color_indexes

            for (const change of this.pendingCellChanges) {
                this.cellColorIndexes[change.yIndex * this.size + change.xIndex] = change.colorIndex
            }
        }).catch(error => {
            console.error('Failed to update PixelMap', error)
        }).finally(() => {
            setTimeout(() => this._refreshLoop(), this.refreshInterval)
        })
    }

    setCellColor(xIndex, yIndex, color) {
        const colorIndex = this.colors.indexOf(color)
        if (colorIndex === -1) {
            throw new Error('Invalid color')
        }

        const changeId = Date.now()
        this.pendingCellChanges.push({changeId, xIndex, yIndex, colorIndex})
        this.cellColorIndexes[yIndex * this.size + xIndex] = colorIndex

        const url = this.setEndpoint +
            '?x_index=' + encodeURIComponent(xIndex) +
            '&y_index=' + encodeURIComponent(yIndex) +
            '&color=' + encodeURIComponent(color)
        fetch(url, {
            method: 'POST'
        }).finally(() => {
            const changeIndex = this.pendingCellChanges.findIndex(change => change.changeId === changeId)
            if (changeIndex !== -1) {
                this.pendingCellChanges.splice(changeIndex, 1)
            }
        })
    }
}
