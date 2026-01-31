import { observer } from "mobx-react-lite";
import { useState } from "react";
import { timelineStore } from "../stores";
import { api } from "../api/client";

export const TimeRangeSelector = observer(function TimeRangeSelector() {
  const [isReextracting, setIsReextracting] = useState(false);
  const [isReextractingAll, setIsReextractingAll] = useState(false);

  const handleReextract = async () => {
    setIsReextracting(true);
    try {
      await api.invalidateExtractions(
        timelineStore.rangeStart.getTime(),
        timelineStore.rangeEnd.getTime()
      );
      await timelineStore.loadData();
    } finally {
      setIsReextracting(false);
    }
  };

  const handleReextractAll = async () => {
    if (!confirm("This will clear ALL extraction cache and re-process everything. This may take a while. Continue?")) {
      return;
    }
    setIsReextractingAll(true);
    try {
      // Invalidate from epoch to far future
      await api.invalidateExtractions(0, Date.now() + 365 * 24 * 60 * 60 * 1000);
      await timelineStore.loadData();
    } finally {
      setIsReextractingAll(false);
    }
  };

  const handleStartChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const date = new Date(e.target.value);
    if (!isNaN(date.getTime())) {
      timelineStore.setTimeRange(date, timelineStore.rangeEnd);
    }
  };

  const handleEndChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const date = new Date(e.target.value);
    if (!isNaN(date.getTime())) {
      timelineStore.setTimeRange(timelineStore.rangeStart, date);
    }
  };

  const formatDateTimeLocal = (date: Date) => {
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
  };

  return (
    <div className="flex flex-wrap items-center gap-4">
      {/* Navigation buttons */}
      <div className="flex items-center gap-1">
        <button
          onClick={() => timelineStore.navigateBack()}
          className="p-1.5 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
          title={`Previous ${timelineStore.navigationStepLabel}`}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <button
          onClick={() => timelineStore.navigateForward()}
          disabled={!timelineStore.canNavigateForward}
          className="p-1.5 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
          title={`Next ${timelineStore.navigationStepLabel}`}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>
      </div>

      {/* Date inputs */}
      <div className="flex items-center gap-2">
        <label className="text-sm text-gray-600 dark:text-gray-400">From:</label>
        <input
          type="datetime-local"
          value={formatDateTimeLocal(timelineStore.rangeStart)}
          onChange={handleStartChange}
          className="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
        />
      </div>
      <div className="flex items-center gap-2">
        <label className="text-sm text-gray-600 dark:text-gray-400">To:</label>
        <input
          type="datetime-local"
          value={formatDateTimeLocal(timelineStore.rangeEnd)}
          onChange={handleEndChange}
          className="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
        />
      </div>

      {/* Preset buttons */}
      <div className="flex gap-2">
        <button
          onClick={() => timelineStore.setToday()}
          className="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
        >
          Today
        </button>
        <button
          onClick={() => timelineStore.setLast24Hours()}
          className="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
        >
          Last 24h
        </button>
        <button
          onClick={() => timelineStore.setLastWeek()}
          className="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
        >
          Last 7 days
        </button>
      </div>

      {/* Re-extract buttons */}
      <div className="flex gap-2">
        <button
          onClick={handleReextract}
          disabled={isReextracting || isReextractingAll || timelineStore.isLoading}
          className="px-3 py-1.5 text-sm text-orange-700 dark:text-orange-300 bg-orange-100 dark:bg-orange-900/30 hover:bg-orange-200 dark:hover:bg-orange-900/50 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-1.5"
          title="Re-extract events in current view with current rules"
        >
          {isReextracting ? (
            <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
          ) : (
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          )}
          Re-extract
        </button>
        <button
          onClick={handleReextractAll}
          disabled={isReextracting || isReextractingAll || timelineStore.isLoading}
          className="px-3 py-1.5 text-sm text-red-700 dark:text-red-300 bg-red-100 dark:bg-red-900/30 hover:bg-red-200 dark:hover:bg-red-900/50 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-1.5"
          title="Re-extract ALL events with current rules (clears entire cache)"
        >
          {isReextractingAll ? (
            <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
          ) : (
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          )}
          Re-extract All
        </button>
      </div>
    </div>
  );
});
