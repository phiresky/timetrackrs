import { createFileRoute, useNavigate, useSearch } from "@tanstack/react-router";
import { observer } from "mobx-react-lite";
import { useEffect } from "react";
import { timelineStore } from "../stores";
import { TimeRangeSelector, TagBreakdown, Timeline, CompactTimeline, TimelineLegend } from "../components";

// Search params type - using ISO date strings for readability
interface TimelineSearchParams {
  from?: string;
  to?: string;
}

const HomePage = observer(function HomePage() {
  const navigate = useNavigate({ from: "/" });
  const search = useSearch({ from: "/" }) as TimelineSearchParams;

  // Sync URL params to store on mount and when URL changes
  useEffect(() => {
    if (search.from && search.to) {
      // URL has time range params - use them
      const fromDate = new Date(search.from);
      const toDate = new Date(search.to);
      if (!isNaN(fromDate.getTime()) && !isNaN(toDate.getTime())) {
        // Only update if different from current store values
        if (
          fromDate.getTime() !== timelineStore.rangeStart.getTime() ||
          toDate.getTime() !== timelineStore.rangeEnd.getTime()
        ) {
          timelineStore.setTimeRange(fromDate, toDate);
        } else {
          // Same range, just load data
          timelineStore.loadData();
        }
      } else {
        timelineStore.loadData();
      }
    } else {
      // No URL params - update URL with current store values
      navigate({
        search: {
          from: timelineStore.rangeStart.toISOString(),
          to: timelineStore.rangeEnd.toISOString(),
        },
        replace: true,
      });
      timelineStore.loadData();
    }
  }, [search.from, search.to]);

  // Sync store changes to URL
  useEffect(() => {
    const storeFromISO = timelineStore.rangeStart.toISOString();
    const storeToISO = timelineStore.rangeEnd.toISOString();

    // Update URL if store values differ from URL
    if (storeFromISO !== search.from || storeToISO !== search.to) {
      navigate({
        search: {
          from: storeFromISO,
          to: storeToISO,
        },
        replace: true,
      });
    }
  }, [timelineStore.rangeStart, timelineStore.rangeEnd]);

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
          Activity Timeline
        </h1>
        <TimeRangeSelector />
      </div>

      {/* Error display */}
      {timelineStore.error && (
        <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm text-red-700 dark:text-red-300">
            {timelineStore.error}
          </p>
        </div>
      )}

      {/* Main content grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Timeline section */}
        <div className="lg:col-span-2 space-y-6">
          {/* Compact overview */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Overview
              </h2>
              <TimelineLegend />
            </div>
            <CompactTimeline />
            <div className="mt-2 flex justify-between text-xs text-gray-500 dark:text-gray-400">
              <span>
                {timelineStore.rangeStart.toLocaleTimeString(undefined, {
                  hour: "2-digit",
                  minute: "2-digit",
                })}
              </span>
              <span>
                {timelineStore.rangeEnd.toLocaleTimeString(undefined, {
                  hour: "2-digit",
                  minute: "2-digit",
                })}
              </span>
            </div>
          </div>

          {/* Detailed timeline */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
            <h2 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              Hourly Breakdown
            </h2>
            <Timeline />
          </div>
        </div>

        {/* Sidebar - Tag breakdown */}
        <div className="lg:col-span-1">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 sticky top-20">
            <TagBreakdown />
          </div>
        </div>
      </div>
    </div>
  );
});

export const Route = createFileRoute("/")({
  component: HomePage,
  validateSearch: (search: Record<string, unknown>): TimelineSearchParams => {
    return {
      from: typeof search.from === "string" ? search.from : undefined,
      to: typeof search.to === "string" ? search.to : undefined,
    };
  },
});
