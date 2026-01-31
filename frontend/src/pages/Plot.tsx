import { useEffect } from 'react'
import { observer } from 'mobx-react-lite'
import {
  AreaChart,
  BarChart,
  Area,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts'
import { plotStore, type BinSize, type ChartType } from '../stores/plotStore'
import { getColorForCategory } from '../lib/categoryColors'
import { formatDuration } from '../lib/formatDuration'

const binSizeOptions: { value: BinSize; label: string }[] = [
  { value: 'hourly', label: 'Hourly' },
  { value: 'daily', label: 'Daily' },
  { value: 'weekly', label: 'Weekly' },
]

const chartTypeOptions: { value: ChartType; label: string }[] = [
  { value: 'area', label: 'Area' },
  { value: 'bar', label: 'Bar' },
]

const quickRangeOptions = [
  { value: 'day' as const, label: 'Today' },
  { value: 'week' as const, label: 'Week' },
  { value: 'month' as const, label: 'Month' },
]

export const Plot = observer(function Plot() {
  useEffect(() => {
    void plotStore.initialize()
  }, [])

  const {
    plotData,
    allTagValues,
    tagValueBreakdown,
    totalTime,
    isLoading,
    error,
    selectedTag,
    knownTags,
    binSize,
    chartType,
    rangeLabel,
  } = plotStore

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Plot</h1>
        <p className="mt-2 text-gray-600">
          Visualize your time tracking data over time
        </p>
      </div>

      {/* Controls */}
      <div className="bg-white rounded-lg shadow-lg p-4 mb-6">
        <div className="flex flex-wrap items-center gap-4">
          {/* Quick Range Selector */}
          <div className="flex items-center gap-2">
            {quickRangeOptions.map((option) => (
              <button
                key={option.value}
                onClick={() => plotStore.setQuickRange(option.value)}
                className="px-4 py-2 text-sm font-medium rounded-md transition-colors bg-white text-gray-700 hover:bg-gray-100 border border-gray-300"
              >
                {option.label}
              </button>
            ))}
          </div>

          {/* Navigation */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => plotStore.navigatePrevious()}
              className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
              aria-label="Previous"
            >
              <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>

            <span className="text-sm font-medium text-gray-900 min-w-48 text-center">
              {rangeLabel}
            </span>

            <button
              onClick={() => plotStore.navigateNext()}
              className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
              aria-label="Next"
            >
              <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>

          {/* Tag Filter */}
          <div className="flex items-center gap-2">
            <label className="text-sm font-medium text-gray-700">Tag:</label>
            <select
              value={selectedTag}
              onChange={(e) => plotStore.setSelectedTag(e.target.value)}
              className="px-3 py-2 text-sm border border-gray-300 rounded-md bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
            >
              {knownTags.length > 0 ? (
                knownTags.map((tag) => (
                  <option key={tag} value={tag}>
                    {tag}
                  </option>
                ))
              ) : (
                <option value="category">category</option>
              )}
            </select>
          </div>

          {/* Bin Size */}
          <div className="flex items-center gap-2">
            <label className="text-sm font-medium text-gray-700">Bin:</label>
            <div className="flex rounded-md border border-gray-300 overflow-hidden">
              {binSizeOptions.map((option) => (
                <button
                  key={option.value}
                  onClick={() => plotStore.setBinSize(option.value)}
                  className={`px-3 py-1.5 text-sm font-medium transition-colors ${
                    binSize === option.value
                      ? 'bg-indigo-600 text-white'
                      : 'bg-white text-gray-700 hover:bg-gray-100'
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          {/* Chart Type */}
          <div className="flex items-center gap-2">
            <label className="text-sm font-medium text-gray-700">Chart:</label>
            <div className="flex rounded-md border border-gray-300 overflow-hidden">
              {chartTypeOptions.map((option) => (
                <button
                  key={option.value}
                  onClick={() => plotStore.setChartType(option.value)}
                  className={`px-3 py-1.5 text-sm font-medium transition-colors ${
                    chartType === option.value
                      ? 'bg-indigo-600 text-white'
                      : 'bg-white text-gray-700 hover:bg-gray-100'
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Error State */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
          <p className="text-red-800">{error}</p>
        </div>
      )}

      {/* Chart */}
      <div className="bg-white rounded-lg shadow-lg p-6 mb-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">
          Time by {selectedTag}
        </h2>

        {isLoading ? (
          <div className="h-96 flex items-center justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" />
          </div>
        ) : plotData.length === 0 || allTagValues.length === 0 ? (
          <div className="h-96 flex items-center justify-center text-gray-500">
            <p>No data available for this time range</p>
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={384}>
            {chartType === 'area' ? (
              <AreaChart data={plotData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                  dataKey="date"
                  stroke="#6b7280"
                  tick={{ fill: '#6b7280', fontSize: 12 }}
                  angle={binSize === 'hourly' ? -45 : 0}
                  textAnchor={binSize === 'hourly' ? 'end' : 'middle'}
                  height={binSize === 'hourly' ? 80 : 30}
                />
                <YAxis
                  stroke="#6b7280"
                  tick={{ fill: '#6b7280' }}
                  tickFormatter={(value: number) => `${value.toFixed(1)}h`}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#1f2937',
                    border: 'none',
                    borderRadius: '0.5rem',
                    color: '#fff',
                  }}
                  formatter={(value) => [`${Number(value).toFixed(2)}h`, '']}
                />
                <Legend />
                {allTagValues.map((tagValue, index) => (
                  <Area
                    key={tagValue}
                    type="monotone"
                    dataKey={tagValue}
                    stackId="1"
                    stroke={getColorForCategory(tagValue, index)}
                    fill={getColorForCategory(tagValue, index)}
                    fillOpacity={0.6}
                  />
                ))}
              </AreaChart>
            ) : (
              <BarChart data={plotData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                  dataKey="date"
                  stroke="#6b7280"
                  tick={{ fill: '#6b7280', fontSize: 12 }}
                  angle={binSize === 'hourly' ? -45 : 0}
                  textAnchor={binSize === 'hourly' ? 'end' : 'middle'}
                  height={binSize === 'hourly' ? 80 : 30}
                />
                <YAxis
                  stroke="#6b7280"
                  tick={{ fill: '#6b7280' }}
                  tickFormatter={(value: number) => `${value.toFixed(1)}h`}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#1f2937',
                    border: 'none',
                    borderRadius: '0.5rem',
                    color: '#fff',
                  }}
                  formatter={(value) => [`${Number(value).toFixed(2)}h`, '']}
                />
                <Legend />
                {allTagValues.map((tagValue, index) => (
                  <Bar
                    key={tagValue}
                    dataKey={tagValue}
                    stackId="1"
                    fill={getColorForCategory(tagValue, index)}
                  />
                ))}
              </BarChart>
            )}
          </ResponsiveContainer>
        )}
      </div>

      {/* Legend with Totals */}
      <div className="bg-white rounded-lg shadow-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-gray-900">Summary</h2>
          <span className="text-lg font-medium text-gray-700">
            Total: {formatDuration(totalTime)}
          </span>
        </div>

        {isLoading ? (
          <div className="h-32 flex items-center justify-center">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600" />
          </div>
        ) : tagValueBreakdown.length === 0 ? (
          <p className="text-gray-500">No data available</p>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
            {tagValueBreakdown.map((item) => (
              <div
                key={item.name}
                className="flex items-center gap-3 p-3 bg-gray-50 rounded-lg"
              >
                <div
                  className="w-4 h-4 rounded-full shrink-0"
                  style={{ backgroundColor: item.color }}
                />
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium text-gray-900 truncate">
                    {item.name}
                  </p>
                  <p className="text-sm text-gray-500">
                    {formatDuration(item.duration)}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
})
