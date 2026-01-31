import { makeAutoObservable, runInAction } from 'mobx'
import { Temporal } from '@js-temporal/polyfill'
import { getTimeRange } from '../lib/api'
import { getColorForCategory } from '../lib/categoryColors'
import type { SingleExtractedChunk } from '../server'

export type TimeRangeType = 'day' | 'week' | 'month'

export interface CategoryData {
  name: string
  duration: number
  color: string
}

export interface HistoryDataPoint {
  date: string
  timestamp: number
  [category: string]: string | number
}

class DashboardStore {
  rangeType: TimeRangeType = 'week'
  currentDate: Temporal.PlainDate = Temporal.Now.plainDateISO()
  chunks: SingleExtractedChunk[] = []
  isLoading = false
  error: string | null = null
  private readonly timezone = Temporal.Now.timeZoneId()

  constructor() {
    makeAutoObservable(this)
  }

  get rangeStart(): Temporal.PlainDate {
    switch (this.rangeType) {
      case 'day':
        return this.currentDate
      case 'week':
        return this.currentDate.subtract({ days: this.currentDate.dayOfWeek - 1 })
      case 'month':
        return this.currentDate.with({ day: 1 })
    }
  }

  get rangeEnd(): Temporal.PlainDate {
    switch (this.rangeType) {
      case 'day':
        return this.currentDate.add({ days: 1 })
      case 'week':
        return this.rangeStart.add({ weeks: 1 })
      case 'month':
        return this.rangeStart.add({ months: 1 })
    }
  }

  get rangeLabel(): string {
    const start = this.rangeStart
    const end = this.rangeEnd.subtract({ days: 1 })
    
    switch (this.rangeType) {
      case 'day':
        return start.toLocaleString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' })
      case 'week':
        return `${start.toLocaleString('en-US', { month: 'short', day: 'numeric' })} - ${end.toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}`
      case 'month':
        return start.toLocaleString('en-US', { month: 'long', year: 'numeric' })
    }
  }

  get totalTrackedTime(): number {
    // Calculate total time based on chunk time spans
    let total = 0
    for (const chunk of this.chunks) {
      const chunkDuration = chunk.to_exclusive.epochMilliseconds - chunk.from.epochMilliseconds
      total += chunkDuration
    }
    return total
  }

  get timeOnComputer(): number {
    let total = 0
    for (const chunk of this.chunks) {
      for (const [tag, value, duration] of chunk.tags) {
        if (tag === 'use-device' && value === 'computer') {
          total += duration
        }
      }
    }
    return total
  }

  get categoryBreakdown(): CategoryData[] {
    const categoryMap = new Map<string, number>()
    
    for (const chunk of this.chunks) {
      for (const [tag, value, duration] of chunk.tags) {
        if (tag === 'category') {
          const current = categoryMap.get(value) || 0
          categoryMap.set(value, current + duration)
        }
      }
    }

    const categories = Array.from(categoryMap.entries())
      .map(([name, duration], index) => ({
        name,
        duration,
        color: getColorForCategory(name, index),
      }))
      .sort((a, b) => b.duration - a.duration)

    return categories
  }

  get totalCategoryTime(): number {
    return this.categoryBreakdown.reduce((sum, cat) => sum + cat.duration, 0)
  }

  get uncategorizedPercentage(): number {
    if (this.timeOnComputer === 0) return 0
    const categorized = this.totalCategoryTime
    const uncategorized = Math.max(0, this.timeOnComputer - categorized)
    return (uncategorized / this.timeOnComputer) * 100
  }

  get productivityPercentage(): number {
    const totalCategory = this.totalCategoryTime
    if (totalCategory === 0) return 0
    
    const productivity = this.categoryBreakdown
      .filter(cat => cat.name.startsWith('Productivity'))
      .reduce((sum, cat) => sum + cat.duration, 0)
    
    return (productivity / totalCategory) * 100
  }

  get historyData(): HistoryDataPoint[] {
    const dataByDate = new Map<string, Map<string, number>>()
    
    for (const chunk of this.chunks) {
      const date = Temporal.Instant.fromEpochMilliseconds(
        chunk.from.epochMilliseconds
      ).toZonedDateTimeISO(this.timezone).toPlainDate().toString()
      
      if (!dataByDate.has(date)) {
        dataByDate.set(date, new Map())
      }
      
      const dateData = dataByDate.get(date)!
      for (const [tag, value, duration] of chunk.tags) {
        if (tag === 'category') {
          const current = dateData.get(value) || 0
          dateData.set(value, current + duration)
        }
      }
    }

    const allCategories = new Set<string>()
    for (const dateData of dataByDate.values()) {
      for (const category of dateData.keys()) {
        allCategories.add(category)
      }
    }

    const result: HistoryDataPoint[] = []
    let currentDate = this.rangeStart
    
    while (Temporal.PlainDate.compare(currentDate, this.rangeEnd) < 0) {
      const dateStr = currentDate.toString()
      const dateData = dataByDate.get(dateStr) || new Map()
      
      const point: HistoryDataPoint = {
        date: currentDate.toLocaleString('en-US', { month: 'short', day: 'numeric' }),
        timestamp: currentDate.toZonedDateTime(this.timezone).epochMilliseconds,
      }
      
      for (const category of allCategories) {
        point[category] = (dateData.get(category) || 0) / (1000 * 60 * 60) // Convert to hours
      }
      
      result.push(point)
      currentDate = currentDate.add({ days: 1 })
    }
    
    return result
  }

  get allCategories(): string[] {
    const categories = new Set<string>()
    for (const chunk of this.chunks) {
      for (const [tag, value] of chunk.tags) {
        if (tag === 'category') {
          categories.add(value)
        }
      }
    }
    return Array.from(categories).sort()
  }

  setRangeType(type: TimeRangeType) {
    this.rangeType = type
    void this.fetchData()
  }

  navigatePrevious() {
    switch (this.rangeType) {
      case 'day':
        this.currentDate = this.currentDate.subtract({ days: 1 })
        break
      case 'week':
        this.currentDate = this.currentDate.subtract({ weeks: 1 })
        break
      case 'month':
        this.currentDate = this.currentDate.subtract({ months: 1 })
        break
    }
    void this.fetchData()
  }

  navigateNext() {
    switch (this.rangeType) {
      case 'day':
        this.currentDate = this.currentDate.add({ days: 1 })
        break
      case 'week':
        this.currentDate = this.currentDate.add({ weeks: 1 })
        break
      case 'month':
        this.currentDate = this.currentDate.add({ months: 1 })
        break
    }
    void this.fetchData()
  }

  navigateToday() {
    this.currentDate = Temporal.Now.plainDateISO()
    void this.fetchData()
  }

  async fetchData() {
    this.isLoading = true
    this.error = null

    try {
      const after = Temporal.Instant.fromEpochMilliseconds(
        this.rangeStart.toZonedDateTime(this.timezone).epochMilliseconds
      )
      const before = Temporal.Instant.fromEpochMilliseconds(
        this.rangeEnd.toZonedDateTime(this.timezone).epochMilliseconds
      )

      const data = await getTimeRange({
        after,
        before,
        tag: null,
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
}

export const dashboardStore = new DashboardStore()
