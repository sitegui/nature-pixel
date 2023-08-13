// A wrapper around a matrix of colored pixels that continuously update itself from the server
class PixelMap {
    constructor() {
        this.size = 100
        this.colors = ['white']
        this.cellColorIndexes = new Array(this.size * this.size).fill(0)
        this.versionId = null

        this.endpoint = '/api/map'
        this.refreshInterval = 500
        this.longPoolingTimeout = 30000

        this._refreshLoop()
    }

    _refreshLoop() {
        const abortController = new AbortController()
        setTimeout(() => abortController.abort(), this.longPoolingTimeout)

        const lastVersionId = this.versionId ? `?last_version_id=${encodeURIComponent(this.versionId)}` : ''
        const url = `${this.endpoint}${lastVersionId}`
        fetch(url, {
            signal: abortController.signal
        }).then(response => {
            return response.json()
        }).then(response => {
            this.versionId = response.version_id
            this.size = response.size
            this.colors = response.colors
            this.cellColorIndexes = response.cell_color_indexes
        }).catch(error => {
            console.error('Failed to update PixelMap', error)
        }).finally(() => {
            setTimeout(() => this._refreshLoop(), this.refreshInterval)
        })
    }
}