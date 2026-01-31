import { makeAutoObservable, runInAction } from "mobx";
import { api } from "../api/client";
import type {
  SingleExtractedChunk,
  SingleExtractedEventWithRaw,
  Tags,
} from "../api/types";
import { getUnixMs } from "../api/types";

// Helper to aggregate tags from chunks
export interface TagAggregate {
  tag: string;
  value: string;
  totalMs: number;
  count: number;
}

function aggregateTags(chunks: SingleExtractedChunk[]): TagAggregate[] {
  const map = new Map<string, TagAggregate>();

  for (const chunk of chunks) {
    const durationMs = getUnixMs(chunk.to_exclusive) - getUnixMs(chunk.from);
    for (const [tag, value, count] of chunk.tags) {
      const key = `${tag}:${value}`;
      const existing = map.get(key);
      if (existing) {
        existing.totalMs += durationMs;
        existing.count += count;
      } else {
        map.set(key, { tag, value, totalMs: durationMs, count });
      }
    }
  }

  return Array.from(map.values()).sort((a, b) => b.totalMs - a.totalMs);
}

// Get primary category from tags
export function getPrimaryCategory(tags: Tags): string {
  const category = tags["category"]?.[0];
  if (category) return category;
  const software = tags["software-name"]?.[0];
  if (software) return software;
  return "Unknown";
}

// Get display title for an event
export function getEventTitle(tags: Tags): string {
  // Priority: window title > browse title > software name > category
  const windowTitle = tags["software-window-title"]?.[0];
  const browseTitle = tags["browse-title"]?.[0];
  const software = tags["software-name"]?.[0];
  const category = tags["category"]?.[0];

  return browseTitle || windowTitle || software || category || "Unknown";
}

// Maximum time range: 31 days
const MAX_RANGE_MS = 31 * 24 * 60 * 60 * 1000;

class TimelineStore {
  // Time range state
  rangeStart: Date = new Date(Date.now() - 24 * 60 * 60 * 1000); // 24 hours ago
  rangeEnd: Date = new Date();

  // Data state
  chunks: SingleExtractedChunk[] = [];
  tagAggregates: TagAggregate[] = [];
  selectedEventIds: string[] = [];
  selectedEvents: SingleExtractedEventWithRaw[] = [];

  // Loading state
  isLoading = false;
  isLoadingEvents = false;
  error: string | null = null;

  // Filter state
  selectedTagFilter: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  setTimeRange(start: Date, end: Date) {
    // Enforce maximum range of 31 days
    const rangeMs = end.getTime() - start.getTime();
    if (rangeMs > MAX_RANGE_MS) {
      // Keep end date, adjust start to be 31 days before
      start = new Date(end.getTime() - MAX_RANGE_MS);
    }
    this.rangeStart = start;
    this.rangeEnd = end;
    this.loadData();
  }

  setToday() {
    const now = new Date();
    const startOfDay = new Date(now);
    startOfDay.setHours(0, 0, 0, 0);
    this.setTimeRange(startOfDay, now);
  }

  setLast24Hours() {
    const now = new Date();
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);
    this.setTimeRange(yesterday, now);
  }

  setLastWeek() {
    const now = new Date();
    const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
    this.setTimeRange(weekAgo, now);
  }

  // Get the duration of the current range in milliseconds
  get rangeDurationMs(): number {
    return this.rangeEnd.getTime() - this.rangeStart.getTime();
  }

  // Determine the navigation mode based on range duration
  get navigationMode(): "day" | "week" | "period" {
    const durationHours = this.rangeDurationMs / (1000 * 60 * 60);
    if (durationHours <= 25) return "day";
    if (durationHours <= 24 * 8) return "week";
    return "period";
  }

  // Navigate backwards with rounding to clean boundaries
  navigateBack() {
    const mode = this.navigationMode;

    if (mode === "day") {
      // Go to previous full day
      const prevDay = new Date(this.rangeStart);
      prevDay.setDate(prevDay.getDate() - 1);
      prevDay.setHours(0, 0, 0, 0);
      const endOfPrevDay = new Date(prevDay);
      endOfPrevDay.setHours(23, 59, 59, 999);
      this.setTimeRange(prevDay, endOfPrevDay);
    } else if (mode === "week") {
      // Go to previous full week (Mon-Sun or just 7 days back)
      const prevWeekStart = new Date(this.rangeStart);
      prevWeekStart.setDate(prevWeekStart.getDate() - 7);
      prevWeekStart.setHours(0, 0, 0, 0);
      const prevWeekEnd = new Date(prevWeekStart);
      prevWeekEnd.setDate(prevWeekEnd.getDate() + 6);
      prevWeekEnd.setHours(23, 59, 59, 999);
      this.setTimeRange(prevWeekStart, prevWeekEnd);
    } else {
      // Just shift by duration for custom periods
      const duration = this.rangeDurationMs;
      const newStart = new Date(this.rangeStart.getTime() - duration);
      const newEnd = new Date(this.rangeEnd.getTime() - duration);
      this.setTimeRange(newStart, newEnd);
    }
  }

  // Navigate forwards with rounding to clean boundaries
  navigateForward() {
    const mode = this.navigationMode;
    const now = new Date();

    if (mode === "day") {
      // Go to next day
      const nextDay = new Date(this.rangeStart);
      nextDay.setDate(nextDay.getDate() + 1);
      nextDay.setHours(0, 0, 0, 0);

      // If next day is today, show start of today to now
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      if (nextDay.getTime() >= today.getTime()) {
        this.setTimeRange(today, now);
      } else {
        const endOfNextDay = new Date(nextDay);
        endOfNextDay.setHours(23, 59, 59, 999);
        this.setTimeRange(nextDay, endOfNextDay);
      }
    } else if (mode === "week") {
      // Go to next week
      const nextWeekStart = new Date(this.rangeStart);
      nextWeekStart.setDate(nextWeekStart.getDate() + 7);
      nextWeekStart.setHours(0, 0, 0, 0);

      // If this week includes today, end at now
      const nextWeekEnd = new Date(nextWeekStart);
      nextWeekEnd.setDate(nextWeekEnd.getDate() + 6);
      nextWeekEnd.setHours(23, 59, 59, 999);

      if (nextWeekEnd.getTime() >= now.getTime()) {
        this.setTimeRange(nextWeekStart, now);
      } else {
        this.setTimeRange(nextWeekStart, nextWeekEnd);
      }
    } else {
      // Just shift by duration for custom periods
      const duration = this.rangeDurationMs;
      const newStart = new Date(this.rangeStart.getTime() + duration);
      let newEnd = new Date(this.rangeEnd.getTime() + duration);
      // Cap at now
      if (newEnd.getTime() > now.getTime()) {
        newEnd = now;
      }
      this.setTimeRange(newStart, newEnd);
    }
  }

  // Check if we can navigate forward (don't go past now)
  get canNavigateForward(): boolean {
    const now = new Date();
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    // For day mode, check if we're already showing today
    if (this.navigationMode === "day") {
      const rangeStartDay = new Date(this.rangeStart);
      rangeStartDay.setHours(0, 0, 0, 0);
      return rangeStartDay.getTime() < today.getTime();
    }

    return this.rangeEnd.getTime() < now.getTime() - 60000; // 1 min buffer
  }

  // Get a human-readable description of the navigation step
  get navigationStepLabel(): string {
    return this.navigationMode;
  }

  setTagFilter(tag: string | null) {
    this.selectedTagFilter = tag;
    this.loadData();
  }

  async loadData() {
    this.isLoading = true;
    this.error = null;

    try {
      const chunks = await api.getTimeRange(
        this.rangeStart.getTime(),
        this.rangeEnd.getTime(),
        this.selectedTagFilter ?? undefined
      );

      runInAction(() => {
        this.chunks = chunks;
        this.tagAggregates = aggregateTags(chunks);
        this.isLoading = false;
      });
    } catch (e) {
      runInAction(() => {
        this.error = e instanceof Error ? e.message : "Unknown error";
        this.isLoading = false;
      });
    }
  }

  async loadEventDetails(eventIds: string[]) {
    if (eventIds.length === 0) return;

    this.isLoadingEvents = true;
    this.selectedEventIds = eventIds;

    try {
      const events = await api.getSingleEvents(eventIds, true, true);
      runInAction(() => {
        this.selectedEvents = events;
        this.isLoadingEvents = false;
      });
    } catch (e) {
      runInAction(() => {
        this.error = e instanceof Error ? e.message : "Unknown error";
        this.isLoadingEvents = false;
      });
    }
  }

  clearSelectedEvents() {
    this.selectedEventIds = [];
    this.selectedEvents = [];
  }

  // Computed: total tracked time in ms
  get totalTrackedMs(): number {
    return this.chunks.reduce(
      (sum, chunk) => sum + (getUnixMs(chunk.to_exclusive) - getUnixMs(chunk.from)),
      0
    );
  }

  // Computed: breakdown by top-level category
  get categoryBreakdown(): Map<string, number> {
    const map = new Map<string, number>();
    for (const agg of this.tagAggregates) {
      if (agg.tag === "category") {
        const topLevel = agg.value.split("/")[0];
        map.set(topLevel, (map.get(topLevel) || 0) + agg.totalMs);
      }
    }
    return map;
  }
}

export const timelineStore = new TimelineStore();
