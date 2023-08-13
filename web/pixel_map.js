// A wrapper around a matrix of colored pixels that continuously update itself from the server
class PixelMap {
    constructor(onload) {
        this.onload = onload

        this.size = 100
        this.colors = ['white']
        this.availableColors = []
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
            const loaded = this.versionId === null
            this.versionId = response.version_id
            this.size = response.size
            this.colors = response.colors
            this.availableColors = response.available_colors
            this.cellColorIndexes = response.cell_color_indexes

            const newPendingCellChanges = []
            for (const change of this.pendingCellChanges) {
                this.cellColorIndexes[change.yIndex * this.size + change.xIndex] = change.colorIndex

                if (change.newVersion === null || change.newVersion > this.versionId) {
                    newPendingCellChanges.push(change)
                }
            }
            this.pendingCellChanges = newPendingCellChanges

            if (loaded) {
                this.onload()
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

        const change = {xIndex, yIndex, colorIndex, newVersion: null}
        this.pendingCellChanges.push(change)
        this.cellColorIndexes[yIndex * this.size + xIndex] = colorIndex

        const url = this.setEndpoint +
            '?x_index=' + encodeURIComponent(xIndex) +
            '&y_index=' + encodeURIComponent(yIndex) +
            '&color=' + encodeURIComponent(color)
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
            }
        })
    }
}
