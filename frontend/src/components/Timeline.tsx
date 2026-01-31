import { observer } from "mobx-react-lite";
import { useMemo } from "react";
import { timelineStore, getEventTitle } from "../stores/timelineStore";
import { formatTime, formatDuration } from "../utils/format";
import type { SingleExtractedChunk } from "../api/types";
import { getUnixMs } from "../api/types";
import { categorizationStore } from "./CategorizationModal";

// Color mapping for categories
const categoryColors: Record<string, { bar: string; dot: string; label: string }> = {
  Productivity: { bar: "bg-green-400 hover:bg-green-500", dot: "bg-green-400", label: "Productivity" },
  Communication: { bar: "bg-blue-400 hover:bg-blue-500", dot: "bg-blue-400", label: "Communication" },
  Entertainment: { bar: "bg-purple-400 hover:bg-purple-500", dot: "bg-purple-400", label: "Entertainment" },
  Reference: { bar: "bg-yellow-400 hover:bg-yellow-500", dot: "bg-yellow-400", label: "Reference" },
  Uncategorized: { bar: "bg-gray-400 hover:bg-gray-500", dot: "bg-gray-400", label: "Uncategorized" },
  System: { bar: "bg-red-400 hover:bg-red-500", dot: "bg-red-400", label: "System" },
};

const defaultColor = { bar: "bg-indigo-400 hover:bg-indigo-500", dot: "bg-indigo-400", label: "Other" };

// Legend component
export function TimelineLegend() {
  const items = Object.values(categoryColors);
  return (
    <div className="flex flex-wrap gap-x-4 gap-y-1">
      {items.map((item) => (
        <div key={item.label} className="flex items-center gap-1.5">
          <div className={`w-3 h-3 rounded ${item.dot}`} />
          <span className="text-xs text-gray-600 dark:text-gray-400">{item.label}</span>
        </div>
      ))}
      <div className="flex items-center gap-1.5">
        <div className={`w-3 h-3 rounded ${defaultColor.dot}`} />
        <span className="text-xs text-gray-600 dark:text-gray-400">{defaultColor.label}</span>
      </div>
    </div>
  );
}

function getChunkColor(chunk: SingleExtractedChunk): string {
  const categoryTag = chunk.tags.find(([tag]) => tag === "category");
  if (categoryTag) {
    const topLevel = categoryTag[1].split("/")[0];
    return categoryColors[topLevel]?.bar || defaultColor.bar;
  }
  return categoryColors["Uncategorized"].bar;
}

function isChunkUncategorized(chunk: SingleExtractedChunk): boolean {
  return !chunk.tags.some(([tag]) => tag === "category");
}

function getChunkTitle(chunk: SingleExtractedChunk): string {
  // Build a simple Tags-like object from the chunk
  const tags: Record<string, string[]> = {};
  for (const [tag, value] of chunk.tags) {
    if (!tags[tag]) tags[tag] = [];
    tags[tag].push(value);
  }
  return getEventTitle(tags);
}

interface TimelineBarProps {
  chunk: SingleExtractedChunk;
  startTime: number;
  endTime: number;
  onClick: () => void;
}

function TimelineBar({ chunk, startTime, endTime, onClick }: TimelineBarProps) {
  const chunkFrom = getUnixMs(chunk.from);
  const chunkTo = getUnixMs(chunk.to_exclusive);
  const totalRange = endTime - startTime;
  const left = ((chunkFrom - startTime) / totalRange) * 100;
  const width = ((chunkTo - chunkFrom) / totalRange) * 100;

  // Don't render tiny chunks
  if (width < 0.1) return null;

  const color = getChunkColor(chunk);
  const title = getChunkTitle(chunk);
  const duration = formatDuration(chunkTo - chunkFrom);

  return (
    <button
      onClick={onClick}
      className={`absolute h-full ${color} rounded transition-colors cursor-pointer group`}
      style={{ left: `${left}%`, width: `${Math.max(width, 0.5)}%` }}
      title={`${title}\n${formatTime(chunkFrom)} - ${formatTime(chunkTo)}\n${duration}`}
    >
      {width > 5 && (
        <span className="absolute inset-0 flex items-center justify-center text-xs text-white font-medium truncate px-1 opacity-0 group-hover:opacity-100 transition-opacity">
          {title}
        </span>
      )}
    </button>
  );
}

// Group chunks by hour for better visualization
interface HourGroup {
  hour: Date;
  chunks: SingleExtractedChunk[];
}

function groupByHour(chunks: SingleExtractedChunk[], startTime: number, endTime: number): HourGroup[] {
  const groups: HourGroup[] = [];
  const hourMs = 60 * 60 * 1000;

  // Create hour boundaries
  const startHour = new Date(startTime);
  startHour.setMinutes(0, 0, 0);

  let currentHour = startHour.getTime();
  while (currentHour < endTime) {
    const nextHour = currentHour + hourMs;
    const hourChunks = chunks.filter(
      (c) => getUnixMs(c.from) < nextHour && getUnixMs(c.to_exclusive) > currentHour
    );

    // Always add the hour, even if empty
    groups.push({
      hour: new Date(currentHour),
      chunks: hourChunks,
    });

    currentHour = nextHour;
  }

  return groups;
}

export const Timeline = observer(function Timeline() {
  const { chunks, rangeStart, rangeEnd, isLoading } = timelineStore;

  const startTime = rangeStart.getTime();
  const endTime = rangeEnd.getTime();

  const hourGroups = useMemo(
    () => groupByHour(chunks, startTime, endTime),
    [chunks, startTime, endTime]
  );

  if (isLoading) {
    return (
      <div className="space-y-2">
        {[...Array(8)].map((_, i) => (
          <div key={i} className="h-8 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
        ))}
      </div>
    );
  }

  if (chunks.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500 dark:text-gray-400">
        <svg
          className="mx-auto h-12 w-12 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <p className="mt-2 text-sm">No activity recorded for this time range</p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {hourGroups.map((group) => {
        const hourStart = group.hour.getTime();
        const hourEnd = hourStart + 60 * 60 * 1000;

        return (
          <div key={hourStart} className="flex items-center gap-3">
            <span className="text-xs text-gray-500 dark:text-gray-400 w-12 flex-shrink-0 text-right">
              {group.hour.toLocaleTimeString(undefined, {
                hour: "2-digit",
                minute: "2-digit",
              })}
            </span>
            <div className="flex-1 h-6 bg-gray-100 dark:bg-gray-800 rounded relative overflow-hidden">
              {group.chunks.map((chunk, idx) => (
                <TimelineBar
                  key={`${getUnixMs(chunk.from)}-${idx}`}
                  chunk={chunk}
                  startTime={hourStart}
                  endTime={hourEnd}
                  onClick={() => {
                    // Extract event IDs from chunk tags (timetrackrs-raw-id)
                    const ids = chunk.tags
                      .filter(([tag]) => tag === "timetrackrs-raw-id")
                      .map(([, value]) => value);
                    if (ids.length > 0) {
                      if (isChunkUncategorized(chunk)) {
                        categorizationStore.open(ids);
                      } else {
                        timelineStore.loadEventDetails(ids);
                      }
                    }
                  }}
                />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
});

// Compact timeline for showing full day at a glance
export const CompactTimeline = observer(function CompactTimeline() {
  const { chunks, rangeStart, rangeEnd, isLoading } = timelineStore;

  const startTime = rangeStart.getTime();
  const endTime = rangeEnd.getTime();

  if (isLoading) {
    return <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />;
  }

  return (
    <div className="h-8 bg-gray-100 dark:bg-gray-800 rounded relative overflow-hidden">
      {chunks.map((chunk, idx) => (
        <TimelineBar
          key={`${getUnixMs(chunk.from)}-${idx}`}
          chunk={chunk}
          startTime={startTime}
          endTime={endTime}
          onClick={() => {
            const ids = chunk.tags
              .filter(([tag]) => tag === "timetrackrs-raw-id")
              .map(([, value]) => value);
            if (ids.length > 0) {
              if (isChunkUncategorized(chunk)) {
                categorizationStore.open(ids);
              } else {
                timelineStore.loadEventDetails(ids);
              }
            }
          }}
        />
      ))}
    </div>
  );
});
