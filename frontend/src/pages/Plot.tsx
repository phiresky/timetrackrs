export function Plot() {
  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Plot</h1>
        <p className="mt-2 text-gray-600">
          Visualize your time tracking data
        </p>
      </div>

      <div className="bg-white rounded-lg shadow-lg p-6">
        <div className="h-96 flex items-center justify-center text-gray-400">
          <p>Chart visualization will be displayed here</p>
        </div>
      </div>
    </div>
  )
}
