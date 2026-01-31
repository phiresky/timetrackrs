interface StatCardProps {
  title: string
  value: string
  subtitle?: string
  icon: string
  color: string
  isLoading?: boolean
}

export function StatCard({ title, value, subtitle, icon, color, isLoading }: StatCardProps) {
  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="p-5">
        <div className="flex items-center">
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-gray-500 truncate">{title}</p>
            {isLoading ? (
              <div className="mt-1 h-9 w-24 bg-gray-200 animate-pulse rounded" />
            ) : (
              <>
                <p className="mt-1 text-3xl font-semibold text-gray-900 truncate">{value}</p>
                {subtitle && (
                  <p className="mt-1 text-sm text-gray-500">{subtitle}</p>
                )}
              </>
            )}
          </div>
          <div
            className={`${color} rounded-full p-3 text-white flex items-center justify-center text-xl flex-shrink-0`}
          >
            {icon}
          </div>
        </div>
      </div>
    </div>
  )
}
