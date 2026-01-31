import { observer } from "mobx-react-lite";
import { timelineStore } from "../stores";
import { formatDateTime, formatDuration } from "../utils/format";
import type { SingleExtractedEventWithRaw, Tags, TagAddReason } from "../api/types";
import { getUnixMs } from "../api/types";

interface TagListProps {
  tags: Tags;
  reasons?: Record<string, TagAddReason> | null;
}

interface FormattedValue {
  display: string;
  isJson: boolean;
  formatted?: string;
}

function formatTagValue(value: unknown): FormattedValue {
  if (value === null || value === undefined) return { display: "", isJson: false };
  if (typeof value === "string") {
    // Check if it's a JSON string
    if (value.startsWith("{") || value.startsWith("[")) {
      try {
        const parsed = JSON.parse(value);
        return {
          display: value.length > 50 ? value.substring(0, 50) + "..." : value,
          isJson: true,
          formatted: JSON.stringify(parsed, null, 2),
        };
      } catch {
        // Not valid JSON, treat as string
      }
    }
    return { display: value, isJson: false };
  }
  if (typeof value === "object") {
    const json = JSON.stringify(value, null, 2);
    return {
      display: json.length > 50 ? json.substring(0, 50) + "..." : json,
      isJson: true,
      formatted: json,
    };
  }
  return { display: String(value), isJson: false };
}

function TagValue({ value, tagKey, reason }: { value: FormattedValue; tagKey: string; reason?: TagAddReason | null }) {
  const isIntrinsic = reason?.type === "IntrinsicTag";

  if (value.isJson && value.formatted) {
    return (
      <details className="w-full">
        <summary
          className={`cursor-pointer inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
            isIntrinsic
              ? "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
              : "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200"
          }`}
        >
          {value.display}
        </summary>
        <pre className="mt-1 p-2 bg-gray-50 dark:bg-gray-900 rounded text-xs overflow-x-auto max-h-48 text-gray-700 dark:text-gray-300">
          {value.formatted}
        </pre>
      </details>
    );
  }

  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
        isIntrinsic
          ? "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
          : "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200"
      }`}
      title={
        reason?.type === "AddedByRule"
          ? `Added by rule matching: ${reason.matched_tags.map((t) => `${t.tag}:${t.value}`).join(", ")}`
          : reason?.type === "IntrinsicTag"
            ? `Intrinsic tag from ${reason.raw_data_type}`
            : undefined
      }
    >
      {value.display}
    </span>
  );
}

function TagList({ tags, reasons }: TagListProps) {
  const entries = Object.entries(tags).flatMap(([key, values]) => {
    // Handle cases where values might be a single value instead of array
    const valueArray = Array.isArray(values) ? values : values ? [values] : [];
    return valueArray.map((value) => ({ key, value: formatTagValue(value) }));
  });

  // Group by tag key
  const grouped = new Map<string, FormattedValue[]>();
  for (const { key, value } of entries) {
    const list = grouped.get(key) || [];
    list.push(value);
    grouped.set(key, list);
  }

  return (
    <div className="space-y-3">
      {Array.from(grouped.entries()).map(([tagKey, values]) => (
        <div key={tagKey}>
          <h4 className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1">
            {tagKey}
          </h4>
          <div className="flex flex-wrap gap-1">
            {values.map((value, idx) => {
              const reasonKey = `${tagKey}:${value.display}`;
              const reason = reasons?.[reasonKey];
              return <TagValue key={idx} value={value} tagKey={tagKey} reason={reason} />;
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

interface EventCardProps {
  event: SingleExtractedEventWithRaw;
}

function EventCard({ event }: EventCardProps) {
  const title =
    event.tags["browse-title"]?.[0] ||
    event.tags["software-window-title"]?.[0] ||
    event.tags["software-name"]?.[0] ||
    "Unknown";

  const category = event.tags["category"]?.[0] || "Uncategorized";
  const software = event.tags["software-name"]?.[0];

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 space-y-4">
      <div className="flex justify-between items-start">
        <div className="flex-1 min-w-0">
          <h3 className="text-sm font-semibold text-gray-900 dark:text-white truncate">
            {title}
          </h3>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
            {formatDateTime(getUnixMs(event.timestamp_unix_ms))} • {formatDuration(event.duration_ms)}
          </p>
        </div>
        <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200">
          {category.split("/")[0]}
        </span>
      </div>

      {software && (
        <p className="text-sm text-gray-600 dark:text-gray-300">
          Application: {software}
        </p>
      )}

      <details className="group">
        <summary className="cursor-pointer text-sm text-indigo-600 dark:text-indigo-400 hover:text-indigo-800 dark:hover:text-indigo-200">
          View all tags ({Object.keys(event.tags).length} categories)
        </summary>
        <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
          <TagList tags={event.tags} reasons={event.tags_reasons} />
        </div>
      </details>

      {event.raw != null && (
        <details className="group">
          <summary className="cursor-pointer text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
            View raw data
          </summary>
          <pre className="mt-2 p-2 bg-gray-50 dark:bg-gray-900 rounded text-xs overflow-x-auto max-h-64">
            {JSON.stringify(event.raw, null, 2) as string}
          </pre>
        </details>
      )}
    </div>
  );
}

export const EventDetail = observer(function EventDetail() {
  const { selectedEvents, isLoadingEvents, selectedEventIds } = timelineStore;

  if (selectedEventIds.length === 0) {
    return null;
  }

  return (
    <div className="fixed inset-y-0 right-0 w-96 bg-gray-50 dark:bg-gray-900 shadow-xl border-l border-gray-200 dark:border-gray-700 overflow-y-auto z-50">
      <div className="sticky top-0 bg-gray-50 dark:bg-gray-900 p-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Event Details
        </h2>
        <button
          onClick={() => timelineStore.clearSelectedEvents()}
          className="p-1 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
        >
          <svg className="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="p-4 space-y-4">
        {isLoadingEvents ? (
          <div className="space-y-4">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="h-32 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse" />
            ))}
          </div>
        ) : selectedEvents.length === 0 ? (
          <p className="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
            No event data available
          </p>
        ) : (
          selectedEvents.map((event) => (
            <EventCard key={event.id} event={event} />
          ))
        )}
      </div>
    </div>
  );
});
