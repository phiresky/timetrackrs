import { observer } from "mobx-react-lite";
import { timelineStore, type TagAggregate } from "../stores/timelineStore";
import { formatDuration } from "../utils/format";

// Color mapping for common categories
const categoryColors: Record<string, string> = {
  Productivity: "bg-green-500",
  Communication: "bg-blue-500",
  Entertainment: "bg-purple-500",
  Reference: "bg-yellow-500",
  Uncategorized: "bg-gray-500",
  System: "bg-red-500",
};

function getCategoryColor(category: string): string {
  const topLevel = category.split("/")[0];
  return categoryColors[topLevel] || "bg-gray-400";
}

interface TagBreakdownItemProps {
  aggregate: TagAggregate;
  maxMs: number;
  onClick: () => void;
  isSelected: boolean;
}

function TagBreakdownItem({ aggregate, maxMs, onClick, isSelected }: TagBreakdownItemProps) {
  const percentage = maxMs > 0 ? (aggregate.totalMs / maxMs) * 100 : 0;
  const color = aggregate.tag === "category" ? getCategoryColor(aggregate.value) : "bg-indigo-500";

  return (
    <button
      onClick={onClick}
      className={`w-full text-left p-2 rounded-lg transition-colors ${
        isSelected
          ? "bg-indigo-100 dark:bg-indigo-900"
          : "hover:bg-gray-50 dark:hover:bg-gray-700"
      }`}
    >
      <div className="flex justify-between items-center mb-1">
        <span className="text-sm font-medium text-gray-900 dark:text-white truncate flex-1">
          {aggregate.value}
        </span>
        <span className="text-xs text-gray-500 dark:text-gray-400 ml-2">
          {formatDuration(aggregate.totalMs)}
        </span>
      </div>
      <div className="h-2 bg-gray-200 dark:bg-gray-600 rounded-full overflow-hidden">
        <div
          className={`h-full ${color} rounded-full transition-all`}
          style={{ width: `${percentage}%` }}
        />
      </div>
    </button>
  );
}

interface TagGroupProps {
  tagName: string;
  aggregates: TagAggregate[];
  maxMs: number;
}

const TagGroup = observer(function TagGroup({ tagName, aggregates, maxMs }: TagGroupProps) {
  const selectedFilter = timelineStore.selectedTagFilter;

  return (
    <div className="space-y-1">
      <h3 className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider px-2">
        {tagName}
      </h3>
      {aggregates.slice(0, 10).map((agg) => (
        <TagBreakdownItem
          key={`${agg.tag}:${agg.value}`}
          aggregate={agg}
          maxMs={maxMs}
          onClick={() => {
            const filter = `${agg.tag}:${agg.value}`;
            if (selectedFilter === filter) {
              timelineStore.setTagFilter(null);
            } else {
              timelineStore.setTagFilter(filter);
            }
          }}
          isSelected={selectedFilter === `${agg.tag}:${agg.value}`}
        />
      ))}
    </div>
  );
});

export const TagBreakdown = observer(function TagBreakdown() {
  const { tagAggregates, totalTrackedMs, isLoading, selectedTagFilter } = timelineStore;

  if (isLoading) {
    return (
      <div className="animate-pulse space-y-3">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="h-12 bg-gray-200 dark:bg-gray-700 rounded-lg" />
        ))}
      </div>
    );
  }

  // Group aggregates by tag name
  const grouped = new Map<string, TagAggregate[]>();
  for (const agg of tagAggregates) {
    const list = grouped.get(agg.tag) || [];
    list.push(agg);
    grouped.set(agg.tag, list);
  }

  // Priority order for display
  const priorityTags = ["category", "software-name", "project", "browse-domain"];
  const sortedTags = Array.from(grouped.keys()).sort((a, b) => {
    const aIdx = priorityTags.indexOf(a);
    const bIdx = priorityTags.indexOf(b);
    if (aIdx >= 0 && bIdx >= 0) return aIdx - bIdx;
    if (aIdx >= 0) return -1;
    if (bIdx >= 0) return 1;
    return a.localeCompare(b);
  });

  // Find max for scaling bars
  const maxMs = Math.max(...tagAggregates.map((a) => a.totalMs), 1);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Time Breakdown
        </h2>
        <span className="text-sm text-gray-500 dark:text-gray-400">
          Total: {formatDuration(totalTrackedMs)}
        </span>
      </div>

      {selectedTagFilter && (
        <div className="flex items-center gap-2 p-2 bg-indigo-50 dark:bg-indigo-900/30 rounded-lg">
          <span className="text-sm text-indigo-700 dark:text-indigo-300">
            Filtering by: {selectedTagFilter}
          </span>
          <button
            onClick={() => timelineStore.setTagFilter(null)}
            className="text-indigo-600 dark:text-indigo-400 hover:text-indigo-800 dark:hover:text-indigo-200"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {sortedTags.slice(0, 5).map((tagName) => (
        <TagGroup
          key={tagName}
          tagName={tagName}
          aggregates={grouped.get(tagName)!}
          maxMs={maxMs}
        />
      ))}

      {tagAggregates.length === 0 && (
        <p className="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
          No data for selected time range
        </p>
      )}
    </div>
  );
});
