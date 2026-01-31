import { observer } from 'mobx-react-lite'
import { dashboardStore, type TimeRangeType } from '../stores/dashboardStore'

const rangeOptions: { value: TimeRangeType; label: string }[] = [
  { value: 'day', label: 'Day' },
  { value: 'week', label: 'Week' },
  { value: 'month', label: 'Month' },
]

export const TimeRangeSelector = observer(function TimeRangeSelector() {
  return (
    <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-6">
      <div className="flex items-center gap-2">
        {rangeOptions.map((option) => (
          <button
            key={option.value}
            onClick={() => dashboardStore.setRangeType(option.value)}
            className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
              dashboardStore.rangeType === option.value
                ? 'bg-indigo-600 text-white'
                : 'bg-white text-gray-700 hover:bg-gray-100 border border-gray-300'
            }`}
          >
            {option.label}
          </button>
        ))}
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={() => dashboardStore.navigatePrevious()}
          className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
          aria-label="Previous"
        >
          <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>

        <button
          onClick={() => dashboardStore.navigateToday()}
          className="px-4 py-2 text-sm font-medium bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-100 transition-colors"
        >
          Today
        </button>

        <button
          onClick={() => dashboardStore.navigateNext()}
          className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
          aria-label="Next"
        >
          <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>

        <span className="ml-4 text-lg font-medium text-gray-900 min-w-48 text-center">
          {dashboardStore.rangeLabel}
        </span>
      </div>
    </div>
  )
})
