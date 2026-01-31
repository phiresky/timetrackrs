import { useEffect } from 'react'
import { observer } from 'mobx-react-lite'
import { Temporal } from '@js-temporal/polyfill'
import { timelineStore, type TimelineBlock } from '../stores/timelineStore'
import { formatDuration } from '../lib/formatDuration'

const TimelineNav = observer(function TimelineNav() {
  return (
    <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-6">
      <div className="flex items-center gap-2">
        <input
          type="date"
          value={timelineStore.currentDate.toString()}
          onChange={(e) => {
            const date = Temporal.PlainDate.from(e.target.value)
            timelineStore.navigateToDate(date)
          }}
          className="px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
        />
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={() => timelineStore.navigatePrevious()}
          className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
          aria-label="Previous day"
        >
          <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>

        <button
          onClick={() => timelineStore.navigateToday()}
          className="px-4 py-2 text-sm font-medium bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-100 transition-colors"
        >
          Today
        </button>

        <button
          onClick={() => timelineStore.navigateNext()}
          className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
          aria-label="Next day"
        >
          <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>

        <span className="ml-4 text-lg font-medium text-gray-900 min-w-48 text-center">
          {timelineStore.dateLabel}
        </span>
      </div>
    </div>
  )
})

const TimelineBlockComponent = observer(function TimelineBlockComponent({
  block,
  onClick,
  isSelected,
}: {
  block: TimelineBlock
  onClick: () => void
  isSelected: boolean
}) {
  const position = timelineStore.getBlockPosition(block)

  return (
    <button
      onClick={onClick}
      className={`absolute h-full rounded transition-all hover:opacity-80 hover:ring-2 hover:ring-offset-1 ${
        isSelected ? 'ring-2 ring-offset-1 ring-indigo-500 z-10' : ''
      }`}
      style={{
        left: position.left,
        width: position.width,
        backgroundColor: block.color,
      }}
      title={`${block.category} - ${formatDuration(block.durationMs)}`}
    />
  )
})

const TimelineVisualization = observer(function TimelineVisualization() {
  const { timelineBlocks, hourMarkers, selectedBlock, isLoading } = timelineStore

  if (isLoading) {
    return (
      <div className="h-32 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" />
      </div>
    )
  }

  if (timelineBlocks.length === 0) {
    return (
      <div className="h-32 flex items-center justify-center text-gray-400">
        <p>No activity data for this day</p>
      </div>
    )
  }

  return (
    <div className="mb-6">
      {/* Hour markers */}
      <div className="relative h-6 mb-2">
        {hourMarkers.map(({ hour, label }) => (
          <div
            key={hour}
            className="absolute text-xs text-gray-500 transform -translate-x-1/2"
            style={{ left: `${(hour / 24) * 100}%` }}
          >
            {label}
          </div>
        ))}
      </div>

      {/* Timeline bar */}
      <div className="relative h-16 bg-gray-100 rounded-lg overflow-hidden">
        {/* Grid lines */}
        {hourMarkers.map(({ hour }) => (
          <div
            key={hour}
            className="absolute h-full w-px bg-gray-200"
            style={{ left: `${(hour / 24) * 100}%` }}
          />
        ))}

        {/* Activity blocks */}
        {timelineBlocks.map((block) => (
          <TimelineBlockComponent
            key={block.id}
            block={block}
            onClick={() => timelineStore.selectBlock(block)}
            isSelected={selectedBlock?.id === block.id}
          />
        ))}
      </div>
    </div>
  )
})

const CategoryLegend = observer(function CategoryLegend() {
  const { timelineBlocks } = timelineStore

  // Get unique categories with their colors
  const categories = new Map<string, { color: string; duration: number }>()
  for (const block of timelineBlocks) {
    const existing = categories.get(block.category)
    if (existing) {
      existing.duration += block.durationMs
    } else {
      categories.set(block.category, { color: block.color, duration: block.durationMs })
    }
  }

  if (categories.size === 0) return null

  return (
    <div className="flex flex-wrap gap-4 mb-6">
      {Array.from(categories.entries())
        .sort((a, b) => b[1].duration - a[1].duration)
        .map(([name, { color, duration }]) => (
          <div key={name} className="flex items-center gap-2">
            <div className="w-4 h-4 rounded" style={{ backgroundColor: color }} />
            <span className="text-sm text-gray-700">
              {name} ({formatDuration(duration)})
            </span>
          </div>
        ))}
    </div>
  )
})

const EventDetails = observer(function EventDetails() {
  const { selectedBlock, isLoadingDetails } = timelineStore

  if (!selectedBlock) {
    return (
      <div className="bg-gray-50 rounded-lg p-6 text-center text-gray-400">
        <p>Click on a timeline block to see details</p>
      </div>
    )
  }

  const formatTime = (instant: Temporal.Instant) => {
    const zdt = instant.toZonedDateTimeISO(Temporal.Now.timeZoneId())
    return zdt.toLocaleString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      second: '2-digit',
      hour12: true,
    })
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6">
      <div className="flex items-center gap-3 mb-4">
        <div className="w-5 h-5 rounded" style={{ backgroundColor: selectedBlock.color }} />
        <h3 className="text-lg font-semibold text-gray-900">{selectedBlock.category}</h3>
        {isLoadingDetails && (
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-indigo-600" />
        )}
      </div>

      <div className="grid grid-cols-2 gap-4 mb-4">
        <div>
          <p className="text-xs font-medium text-gray-500 uppercase">Start Time</p>
          <p className="text-sm text-gray-900">{formatTime(selectedBlock.from)}</p>
        </div>
        <div>
          <p className="text-xs font-medium text-gray-500 uppercase">End Time</p>
          <p className="text-sm text-gray-900">{formatTime(selectedBlock.to)}</p>
        </div>
        <div>
          <p className="text-xs font-medium text-gray-500 uppercase">Duration</p>
          <p className="text-sm text-gray-900">{formatDuration(selectedBlock.durationMs)}</p>
        </div>
      </div>

      {selectedBlock.tags.size > 0 && (
        <div>
          <p className="text-xs font-medium text-gray-500 uppercase mb-2">Tags</p>
          <div className="space-y-2">
            {Array.from(selectedBlock.tags.entries()).map(([tag, values]) => (
              <div key={tag} className="flex flex-wrap items-start gap-2">
                <span className="text-xs font-medium text-gray-600 bg-gray-100 px-2 py-1 rounded">
                  {tag}
                </span>
                <div className="flex flex-wrap gap-1">
                  {values.map((value, idx) => (
                    <span
                      key={idx}
                      className="text-xs text-gray-700 bg-gray-50 px-2 py-1 rounded border border-gray-200"
                    >
                      {value}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
})

export const Timeline = observer(function Timeline() {
  useEffect(() => {
    void timelineStore.fetchData()
  }, [])

  const { error } = timelineStore

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900">Timeline</h1>
        <p className="mt-2 text-gray-600">View your activities over time</p>
      </div>

      <TimelineNav />

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-800">{error}</p>
        </div>
      )}

      <div className="bg-white rounded-lg shadow-lg p-6 mb-6">
        <TimelineVisualization />
        <CategoryLegend />
      </div>

      <div className="bg-white rounded-lg shadow-lg p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Event Details</h2>
        <EventDetails />
      </div>
    </div>
  )
})
