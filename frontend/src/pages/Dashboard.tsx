import { useEffect } from 'react'
import { observer } from 'mobx-react-lite'
import { dashboardStore } from '../stores/dashboardStore'
import { TimeRangeSelector } from '../components/TimeRangeSelector'
import { StatCard } from '../components/StatCard'
import { CategoryHistoryChart } from '../components/CategoryHistoryChart'
import { CategoryPieChart } from '../components/CategoryPieChart'
import { formatDuration, formatPercentage } from '../lib/formatDuration'

export const Dashboard = observer(function Dashboard() {
  useEffect(() => {
    void dashboardStore.fetchData()
  }, [])

  const {
    totalTrackedTime,
    timeOnComputer,
    uncategorizedPercentage,
    productivityPercentage,
    isLoading,
    error,
  } = dashboardStore

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
        <p className="mt-2 text-gray-600">
          Overview of your tracked time and activities
        </p>
      </div>

      <TimeRangeSelector />

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-800">{error}</p>
        </div>
      )}

      {/* Stats cards */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4 mb-8">
        <StatCard
          title="Total tracked time"
          value={formatDuration(totalTrackedTime)}
          icon="📊"
          color="bg-red-500"
          isLoading={isLoading}
        />
        <StatCard
          title="Time on computer"
          value={formatDuration(timeOnComputer)}
          icon="💻"
          color="bg-yellow-500"
          isLoading={isLoading}
        />
        <StatCard
          title="Uncategorized time"
          value={formatPercentage(uncategorizedPercentage)}
          icon="❓"
          color="bg-amber-500"
          isLoading={isLoading}
        />
        <StatCard
          title="Productivity"
          value={formatPercentage(productivityPercentage)}
          icon="📈"
          color="bg-indigo-500"
          isLoading={isLoading}
        />
      </div>

      {/* Main content area */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <div className="bg-gray-800 rounded-lg shadow-lg p-6">
            <div className="mb-4">
              <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide">
                Time spent by category
              </h3>
              <h2 className="text-xl font-bold text-white">History</h2>
            </div>
            <CategoryHistoryChart />
          </div>
        </div>

        <div className="lg:col-span-1">
          <div className="bg-white rounded-lg shadow-lg p-6">
            <div className="mb-4">
              <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                Time spent by category
              </h3>
              <h2 className="text-xl font-bold text-gray-900">Overview</h2>
            </div>
            <CategoryPieChart />
          </div>
        </div>
      </div>
    </div>
  )
})
