export function Dashboard() {
  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
        <p className="mt-2 text-gray-600">
          Overview of your tracked time and activities
        </p>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4 mb-8">
        <StatCard
          title="Total tracked time"
          value="--"
          icon="📊"
          color="bg-red-500"
        />
        <StatCard
          title="Time on computer"
          value="--"
          icon="💻"
          color="bg-yellow-500"
        />
        <StatCard
          title="Uncategorized time"
          value="--"
          icon="❓"
          color="bg-amber-500"
        />
        <StatCard
          title="Productivity"
          value="--"
          icon="📈"
          color="bg-indigo-500"
        />
      </div>

      {/* Main content area */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <div className="bg-gray-800 rounded-lg shadow-lg p-6">
            <div className="flex justify-between items-center mb-4">
              <div>
                <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide">
                  Time spent by category
                </h3>
                <h2 className="text-xl font-bold text-white">History</h2>
              </div>
              <div className="flex space-x-2">
                <button className="px-3 py-1 text-sm bg-indigo-600 text-white rounded-md">
                  Simple
                </button>
                <button className="px-3 py-1 text-sm text-gray-400 hover:text-white rounded-md">
                  Detailed
                </button>
              </div>
            </div>
            <div className="h-64 flex items-center justify-center text-gray-500">
              <p>Chart will be displayed here</p>
            </div>
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
            <div className="h-64 flex items-center justify-center text-gray-400">
              <p>Pie chart will be displayed here</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

interface StatCardProps {
  title: string
  value: string
  icon: string
  color: string
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="p-5">
        <div className="flex items-center">
          <div className="flex-1">
            <p className="text-sm font-medium text-gray-500 truncate">{title}</p>
            <p className="mt-1 text-3xl font-semibold text-gray-900">{value}</p>
          </div>
          <div
            className={`${color} rounded-full p-3 text-white flex items-center justify-center text-xl`}
          >
            {icon}
          </div>
        </div>
      </div>
    </div>
  )
}
