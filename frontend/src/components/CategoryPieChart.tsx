import { observer } from 'mobx-react-lite'
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Legend,
  Tooltip,
} from 'recharts'
import { dashboardStore } from '../stores/dashboardStore'
import { formatDuration } from '../lib/formatDuration'

export const CategoryPieChart = observer(function CategoryPieChart() {
  const { categoryBreakdown, isLoading } = dashboardStore

  if (isLoading) {
    return (
      <div className="h-64 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" />
      </div>
    )
  }

  if (categoryBreakdown.length === 0) {
    return (
      <div className="h-64 flex items-center justify-center text-gray-400">
        <p>No category data available</p>
      </div>
    )
  }

  return (
    <ResponsiveContainer width="100%" height={256}>
      <PieChart>
        <Pie
          data={categoryBreakdown}
          cx="50%"
          cy="50%"
          innerRadius={40}
          outerRadius={80}
          paddingAngle={2}
          dataKey="duration"
          nameKey="name"
          label={({ name, percent }) => `${name} (${((percent ?? 0) * 100).toFixed(0)}%)`}
          labelLine={false}
        >
          {categoryBreakdown.map((entry, index) => (
            <Cell key={`cell-${index}`} fill={entry.color} />
          ))}
        </Pie>
        <Tooltip
          contentStyle={{
            backgroundColor: '#fff',
            border: '1px solid #e5e7eb',
            borderRadius: '0.5rem',
          }}
          formatter={(value) => [formatDuration(Number(value)), 'Time']}
        />
        <Legend
          formatter={(value: string) => (
            <span className="text-gray-700">{value}</span>
          )}
        />
      </PieChart>
    </ResponsiveContainer>
  )
})
