import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { appStore } from '../stores/appStore'

export const ProgressPopup = observer(function ProgressPopup() {
  useEffect(() => {
    appStore.startProgressListener()
    return () => appStore.stopProgressListener()
  }, [])

  const reports = appStore.activeProgressReports

  if (reports.length === 0) {
    return null
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 space-y-2">
      {reports.map((report) => (
        <div
          key={report.call_id}
          className="bg-white rounded-lg shadow-lg p-4 max-w-sm border border-gray-200"
        >
          <div className="text-sm font-medium text-gray-900 mb-2">
            {report.call_desc}
          </div>
          {report.state.map((state, idx) => (
            <div key={idx} className="mb-1">
              <div className="text-xs text-gray-600">{state.desc}</div>
              {state.total !== null && (
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-indigo-600 h-2 rounded-full transition-all duration-300"
                    style={{
                      width: `${(state.current / state.total) * 100}%`,
                    }}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  )
})
