import { makeAutoObservable, runInAction } from 'mobx'
import { Temporal } from '@js-temporal/polyfill'
import { getTimeRange, getSingleEvents } from '../lib/api'
import { getColorForCategory } from '../lib/categoryColors'
import type { SingleExtractedChunk, SingleExtractedEventWithRaw } from '../server'

export interface TimelineBlock {
  id: string
  from: Temporal.Instant
  to: Temporal.Instant
  category: string
  color: string
  tags: Map<string, string[]>
  durationMs: number
}

class TimelineStore {
  currentDate: Temporal.PlainDate = Temporal.Now.plainDateISO()
  chunks: SingleExtractedChunk[] = []
  isLoading = false
  error: string | null = null
  selectedBlock: TimelineBlock | null = null
  selectedEventDetails: SingleExtractedEventWithRaw | null = null
  isLoadingDetails = false

  constructor() {
    makeAutoObservable(this)
  }

  get dayStart(): Temporal.Instant {
    return this.currentDate
      .toZonedDateTime({ timeZone: Temporal.Now.timeZoneId(), plainTime: new Temporal.PlainTime(0, 0, 0) })
      .toInstant()
  }

  get dayEnd(): Temporal.Instant {
    return this.currentDate
      .add({ days: 1 })
      .toZonedDateTime({ timeZone: Temporal.Now.timeZoneId(), plainTime: new Temporal.PlainTime(0, 0, 0) })
      .toInstant()
  }

  get dateLabel(): string {
    return this.currentDate.toLocaleString('en-US', {
      weekday: 'long',
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    })
  }

  get timelineBlocks(): TimelineBlock[] {
    const blocks: TimelineBlock[] = []
    let categoryIndex = 0
    const categoryIndexMap = new Map<string, number>()

    for (const chunk of this.chunks) {
      const tags = new Map<string, string[]>()
      let category = 'Other'

      for (const [tag, value] of chunk.tags) {
        if (!tags.has(tag)) {
          tags.set(tag, [])
        }
        tags.get(tag)!.push(value)

        if (tag === 'category') {
          category = value
        }
      }

      if (!categoryIndexMap.has(category)) {
        categoryIndexMap.set(category, categoryIndex++)
      }

      const durationMs = chunk.to_exclusive.epochMilliseconds - chunk.from.epochMilliseconds

      blocks.push({
        id: `${chunk.from.epochMilliseconds}-${chunk.to_exclusive.epochMilliseconds}`,
        from: chunk.from,
        to: chunk.to_exclusive,
        category,
        color: getColorForCategory(category, categoryIndexMap.get(category)!),
        tags,
        durationMs,
      })
    }

    return blocks
  }

  get hourMarkers(): { hour: number; label: string }[] {
    const markers: { hour: number; label: string }[] = []
    for (let i = 0; i <= 24; i += 2) {
      const hour = i % 24
      const label = hour === 0 ? '12 AM' : hour === 12 ? '12 PM' : hour < 12 ? `${hour} AM` : `${hour - 12} PM`
      markers.push({ hour: i, label })
    }
    return markers
  }

  getBlockPosition(block: TimelineBlock): { left: string; width: string } {
    const dayStartMs = this.dayStart.epochMilliseconds
    const dayEndMs = this.dayEnd.epochMilliseconds
    const dayDurationMs = dayEndMs - dayStartMs

    const blockStartMs = Math.max(block.from.epochMilliseconds, dayStartMs)
    const blockEndMs = Math.min(block.to.epochMilliseconds, dayEndMs)

    const left = ((blockStartMs - dayStartMs) / dayDurationMs) * 100
    const width = ((blockEndMs - blockStartMs) / dayDurationMs) * 100

    return {
      left: `${left}%`,
      width: `${Math.max(width, 0.1)}%`,
    }
  }

  navigatePrevious() {
    this.currentDate = this.currentDate.subtract({ days: 1 })
    this.selectedBlock = null
    this.selectedEventDetails = null
    void this.fetchData()
  }

  navigateNext() {
    this.currentDate = this.currentDate.add({ days: 1 })
    this.selectedBlock = null
    this.selectedEventDetails = null
    void this.fetchData()
  }

  navigateToday() {
    this.currentDate = Temporal.Now.plainDateISO()
    this.selectedBlock = null
    this.selectedEventDetails = null
    void this.fetchData()
  }

  navigateToDate(date: Temporal.PlainDate) {
    this.currentDate = date
    this.selectedBlock = null
    this.selectedEventDetails = null
    void this.fetchData()
  }

  selectBlock(block: TimelineBlock | null) {
    this.selectedBlock = block
    this.selectedEventDetails = null
    if (block) {
      void this.fetchEventDetails(block)
    }
  }

  async fetchEventDetails(block: TimelineBlock) {
    this.isLoadingDetails = true

    try {
      // Use the block's timestamp as an ID - the backend expects specific event IDs
      // For now, we'll just show the block's tag info
      const midpoint = Math.floor(
        (block.from.epochMilliseconds + block.to.epochMilliseconds) / 2
      )
      const id = midpoint.toString()

      const events = await getSingleEvents({
        ids: [id],
        include_raw: true,
        include_reasons: false,
      })

      runInAction(() => {
        if (events.length > 0) {
          this.selectedEventDetails = events[0]
        }
        this.isLoadingDetails = false
      })
    } catch {
      runInAction(() => {
        this.isLoadingDetails = false
      })
    }
  }

  async fetchData() {
    this.isLoading = true
    this.error = null

    try {
      const data = await getTimeRange({
        after: this.dayStart,
        before: this.dayEnd,
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

export const timelineStore = new TimelineStore()
