import { observer } from 'mobx-react-lite'
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts'
import { dashboardStore } from '../stores/dashboardStore'
import { getColorForCategory } from '../lib/categoryColors'

export const CategoryHistoryChart = observer(function CategoryHistoryChart() {
  const { historyData, allCategories, isLoading } = dashboardStore

  if (isLoading) {
    return (
      <div className="h-64 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" />
      </div>
    )
  }

  if (historyData.length === 0 || allCategories.length === 0) {
    return (
      <div className="h-64 flex items-center justify-center text-gray-500">
        <p>No data available for this time range</p>
      </div>
    )
  }

  return (
    <ResponsiveContainer width="100%" height={256}>
      <AreaChart data={historyData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
        <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
        <XAxis 
          dataKey="date" 
          stroke="#9ca3af"
          tick={{ fill: '#9ca3af' }}
        />
        <YAxis 
          stroke="#9ca3af"
          tick={{ fill: '#9ca3af' }}
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
        {allCategories.map((category, index) => (
          <Area
            key={category}
            type="monotone"
            dataKey={category}
            stackId="1"
            stroke={getColorForCategory(category, index)}
            fill={getColorForCategory(category, index)}
            fillOpacity={0.6}
          />
        ))}
      </AreaChart>
    </ResponsiveContainer>
  )
})
