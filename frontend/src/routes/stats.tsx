import { createFileRoute } from "@tanstack/react-router";
import { observer } from "mobx-react-lite";
import { useEffect } from "react";
import { timelineStore } from "../stores";
import { TimeRangeSelector } from "../components";
import { formatDuration, formatPercent } from "../utils/format";

// Color mapping for categories
const categoryColors: Record<string, string> = {
  Productivity: "bg-green-500",
  Communication: "bg-blue-500",
  Entertainment: "bg-purple-500",
  Reference: "bg-yellow-500",
  Uncategorized: "bg-gray-500",
  System: "bg-red-500",
};

const CategoryPieChart = observer(function CategoryPieChart() {
  const { categoryBreakdown, totalTrackedMs } = timelineStore;

  const categories = Array.from(categoryBreakdown.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 6);

  if (categories.length === 0) {
    return (
      <div className="flex items-center justify-center h-48 text-gray-500 dark:text-gray-400">
        No data available
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Stacked bar visualization */}
      <div className="h-8 rounded-lg overflow-hidden flex">
        {categories.map(([category, ms]) => {
          const percent = totalTrackedMs > 0 ? (ms / totalTrackedMs) * 100 : 0;
          const color = categoryColors[category] || "bg-gray-400";

          return (
            <div
              key={category}
              className={`${color} transition-all`}
              style={{ width: `${percent}%` }}
              title={`${category}: ${formatPercent(ms, totalTrackedMs)}`}
            />
          );
        })}
      </div>

      {/* Legend */}
      <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
        {categories.map(([category, ms]) => {
          const color = categoryColors[category] || "bg-gray-400";
          return (
            <div key={category} className="flex items-center gap-2">
              <div className={`w-3 h-3 rounded ${color}`} />
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                  {category}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {formatDuration(ms)} ({formatPercent(ms, totalTrackedMs)})
                </p>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
});

const TopActivities = observer(function TopActivities() {
  const { tagAggregates } = timelineStore;

  // Get top software
  const topSoftware = tagAggregates
    .filter((a) => a.tag === "software-name")
    .slice(0, 5);

  // Get top domains
  const topDomains = tagAggregates
    .filter((a) => a.tag === "browse-domain")
    .slice(0, 5);

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      {/* Top Applications */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
          Top Applications
        </h3>
        <div className="space-y-3">
          {topSoftware.length === 0 ? (
            <p className="text-sm text-gray-500 dark:text-gray-400">No data</p>
          ) : (
            topSoftware.map((item, idx) => (
              <div key={item.value} className="flex items-center gap-3">
                <span className="text-sm font-medium text-gray-400 w-4">
                  {idx + 1}
                </span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-gray-900 dark:text-white truncate">
                    {item.value}
                  </p>
                  <div className="mt-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-indigo-500 rounded-full"
                      style={{
                        width: `${(item.totalMs / (topSoftware[0]?.totalMs || 1)) * 100}%`,
                      }}
                    />
                  </div>
                </div>
                <span className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                  {formatDuration(item.totalMs)}
                </span>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Top Websites */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
          Top Websites
        </h3>
        <div className="space-y-3">
          {topDomains.length === 0 ? (
            <p className="text-sm text-gray-500 dark:text-gray-400">No data</p>
          ) : (
            topDomains.map((item, idx) => (
              <div key={item.value} className="flex items-center gap-3">
                <span className="text-sm font-medium text-gray-400 w-4">
                  {idx + 1}
                </span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-gray-900 dark:text-white truncate">
                    {item.value}
                  </p>
                  <div className="mt-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-500 rounded-full"
                      style={{
                        width: `${(item.totalMs / (topDomains[0]?.totalMs || 1)) * 100}%`,
                      }}
                    />
                  </div>
                </div>
                <span className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                  {formatDuration(item.totalMs)}
                </span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
});

const StatsPage = observer(function StatsPage() {
  useEffect(() => {
    timelineStore.loadData();
  }, []);

  const { totalTrackedMs, isLoading } = timelineStore;

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
          Statistics
        </h1>
        <TimeRangeSelector />
      </div>

      {/* Summary cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Tracked</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {isLoading ? "..." : formatDuration(totalTrackedMs)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">Events</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {isLoading ? "..." : timelineStore.chunks.length}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">Categories</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {isLoading ? "..." : timelineStore.categoryBreakdown.size}
          </p>
        </div>
      </div>

      {/* Category breakdown */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Time by Category
        </h2>
        <CategoryPieChart />
      </div>

      {/* Top activities */}
      <TopActivities />
    </div>
  );
});

export const Route = createFileRoute("/stats")({
  component: StatsPage,
});
