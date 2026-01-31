import { makeAutoObservable, runInAction } from 'mobx'
import { Temporal } from '@js-temporal/polyfill'
import { getTimeRange, getKnownTags } from '../lib/api'
import { getColorForCategory } from '../lib/categoryColors'
import type { SingleExtractedChunk } from '../server'

export type BinSize = 'hourly' | 'daily' | 'weekly'
export type ChartType = 'area' | 'bar'

export interface PlotDataPoint {
  date: string
  timestamp: number
  [tagValue: string]: string | number
}

export interface TagValueData {
  name: string
  duration: number
  color: string
}

class PlotStore {
  startDate: Temporal.PlainDate = Temporal.Now.plainDateISO().subtract({ weeks: 1 })
  endDate: Temporal.PlainDate = Temporal.Now.plainDateISO().add({ days: 1 })
  selectedTag: string = 'category'
  binSize: BinSize = 'daily'
  chartType: ChartType = 'area'
  chunks: SingleExtractedChunk[] = []
  knownTags: string[] = []
  isLoading = false
  error: string | null = null

  constructor() {
    makeAutoObservable(this)
  }

  get rangeLabel(): string {
    const start = this.startDate
    const end = this.endDate.subtract({ days: 1 })
    return `${start.toLocaleString('en-US', { month: 'short', day: 'numeric' })} - ${end.toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}`
  }

  get allTagValues(): string[] {
    const values = new Set<string>()
    for (const chunk of this.chunks) {
      for (const [tag, value] of chunk.tags) {
        if (tag === this.selectedTag) {
          values.add(value)
        }
      }
    }
    return Array.from(values).sort()
  }

  get tagValueBreakdown(): TagValueData[] {
    const valueMap = new Map<string, number>()

    for (const chunk of this.chunks) {
      for (const [tag, value, duration] of chunk.tags) {
        if (tag === this.selectedTag) {
          const current = valueMap.get(value) || 0
          valueMap.set(value, current + duration)
        }
      }
    }

    return Array.from(valueMap.entries())
      .map(([name, duration], index) => ({
        name,
        duration,
        color: getColorForCategory(name, index),
      }))
      .sort((a, b) => b.duration - a.duration)
  }

  get totalTime(): number {
    return this.tagValueBreakdown.reduce((sum, item) => sum + item.duration, 0)
  }

  get plotData(): PlotDataPoint[] {
    const binData = new Map<string, Map<string, number>>()

    for (const chunk of this.chunks) {
      const timestamp = chunk.from.epochMilliseconds
      const binKey = this.getBinKey(timestamp)

      if (!binData.has(binKey)) {
        binData.set(binKey, new Map())
      }

      const bin = binData.get(binKey)!
      for (const [tag, value, duration] of chunk.tags) {
        if (tag === this.selectedTag) {
          const current = bin.get(value) || 0
          bin.set(value, current + duration)
        }
      }
    }

    const allValues = this.allTagValues
    const result: PlotDataPoint[] = []

    const binKeys = this.generateBinKeys()
    for (const binKey of binKeys) {
      const bin = binData.get(binKey) || new Map()

      const point: PlotDataPoint = {
        date: this.formatBinLabel(binKey),
        timestamp: this.binKeyToTimestamp(binKey),
      }

      for (const value of allValues) {
        point[value] = (bin.get(value) || 0) / (1000 * 60 * 60) // Convert to hours
      }

      result.push(point)
    }

    return result
  }

  private getBinKey(timestampMs: number): string {
    const instant = Temporal.Instant.fromEpochMilliseconds(timestampMs)
    const zdt = instant.toZonedDateTimeISO(Temporal.Now.timeZoneId())
    const date = zdt.toPlainDate()

    switch (this.binSize) {
      case 'hourly': {
        const hour = zdt.hour.toString().padStart(2, '0')
        return `${date.toString()}T${hour}`
      }
      case 'daily':
        return date.toString()
      case 'weekly': {
        const weekStart = date.subtract({ days: date.dayOfWeek - 1 })
        return weekStart.toString()
      }
    }
  }

  private generateBinKeys(): string[] {
    const keys: string[] = []
    let current = this.startDate

    switch (this.binSize) {
      case 'hourly': {
        while (Temporal.PlainDate.compare(current, this.endDate) < 0) {
          for (let h = 0; h < 24; h++) {
            const hour = h.toString().padStart(2, '0')
            keys.push(`${current.toString()}T${hour}`)
          }
          current = current.add({ days: 1 })
        }
        break
      }
      case 'daily': {
        while (Temporal.PlainDate.compare(current, this.endDate) < 0) {
          keys.push(current.toString())
          current = current.add({ days: 1 })
        }
        break
      }
      case 'weekly': {
        let weekStart = current.subtract({ days: current.dayOfWeek - 1 })
        while (Temporal.PlainDate.compare(weekStart, this.endDate) < 0) {
          keys.push(weekStart.toString())
          weekStart = weekStart.add({ weeks: 1 })
        }
        break
      }
    }

    return keys
  }

  private formatBinLabel(binKey: string): string {
    switch (this.binSize) {
      case 'hourly': {
        const [datePart, hour] = binKey.split('T')
        const date = Temporal.PlainDate.from(datePart)
        return `${date.toLocaleString('en-US', { month: 'short', day: 'numeric' })} ${hour}:00`
      }
      case 'daily': {
        const date = Temporal.PlainDate.from(binKey)
        return date.toLocaleString('en-US', { month: 'short', day: 'numeric' })
      }
      case 'weekly': {
        const date = Temporal.PlainDate.from(binKey)
        const endDate = date.add({ days: 6 })
        return `${date.toLocaleString('en-US', { month: 'short', day: 'numeric' })} - ${endDate.toLocaleString('en-US', { month: 'short', day: 'numeric' })}`
      }
    }
  }

  private binKeyToTimestamp(binKey: string): number {
    switch (this.binSize) {
      case 'hourly': {
        const [datePart, hour] = binKey.split('T')
        const date = Temporal.PlainDate.from(datePart)
        const plainTime = new Temporal.PlainTime(parseInt(hour), 0)
        return date.toZonedDateTime({
          timeZone: Temporal.Now.timeZoneId(),
          plainTime,
        }).epochMilliseconds
      }
      default: {
        const date = Temporal.PlainDate.from(binKey)
        return date.toZonedDateTime({
          timeZone: Temporal.Now.timeZoneId(),
          plainTime: new Temporal.PlainTime(0, 0, 0),
        }).epochMilliseconds
      }
    }
  }

  setStartDate(date: Temporal.PlainDate) {
    this.startDate = date
    void this.fetchData()
  }

  setEndDate(date: Temporal.PlainDate) {
    this.endDate = date
    void this.fetchData()
  }

  setDateRange(start: Temporal.PlainDate, end: Temporal.PlainDate) {
    this.startDate = start
    this.endDate = end
    void this.fetchData()
  }

  setSelectedTag(tag: string) {
    this.selectedTag = tag
    void this.fetchData()
  }

  setBinSize(size: BinSize) {
    this.binSize = size
  }

  setChartType(type: ChartType) {
    this.chartType = type
  }

  navigatePrevious() {
    const duration = this.endDate.since(this.startDate)
    this.startDate = this.startDate.subtract(duration)
    this.endDate = this.endDate.subtract(duration)
    void this.fetchData()
  }

  navigateNext() {
    const duration = this.endDate.since(this.startDate)
    this.startDate = this.startDate.add(duration)
    this.endDate = this.endDate.add(duration)
    void this.fetchData()
  }

  setQuickRange(range: 'day' | 'week' | 'month') {
    const today = Temporal.Now.plainDateISO()
    switch (range) {
      case 'day':
        this.startDate = today
        this.endDate = today.add({ days: 1 })
        this.binSize = 'hourly'
        break
      case 'week':
        this.startDate = today.subtract({ weeks: 1 })
        this.endDate = today.add({ days: 1 })
        this.binSize = 'daily'
        break
      case 'month':
        this.startDate = today.subtract({ months: 1 })
        this.endDate = today.add({ days: 1 })
        this.binSize = 'daily'
        break
    }
    void this.fetchData()
  }

  async fetchKnownTags() {
    try {
      const tags = await getKnownTags()
      runInAction(() => {
        this.knownTags = tags
      })
    } catch (err) {
      console.error('Failed to fetch known tags:', err)
    }
  }

  async fetchData() {
    this.isLoading = true
    this.error = null

    try {
      const after = this.startDate.toZonedDateTime({
        timeZone: Temporal.Now.timeZoneId(),
        plainTime: new Temporal.PlainTime(0, 0, 0),
      }).toInstant()
      const before = this.endDate.toZonedDateTime({
        timeZone: Temporal.Now.timeZoneId(),
        plainTime: new Temporal.PlainTime(0, 0, 0),
      }).toInstant()

      const data = await getTimeRange({
        after,
        before,
        tag: this.selectedTag,
      })

      runInAction(() => {
        this.chunks = data
        this.isLoading = false
      })
    } catch (err) {
      runInAction(() => {
        this.error = err instanceof Error ? err.message : 'Failed to fetch data'
        this.isLoading = false
      })
    }
  }

  async initialize() {
    await this.fetchKnownTags()
    await this.fetchData()
  }
}

export const plotStore = new PlotStore()
