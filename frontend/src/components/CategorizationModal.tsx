import { observer } from "mobx-react-lite";
import { useState } from "react";
import { makeAutoObservable, runInAction } from "mobx";
import { api } from "../api/client";
import type { SingleExtractedEventWithRaw } from "../api/types";
import { getUnixMs } from "../api/types";
import { timelineStore } from "../stores/timelineStore";

interface TagSelection {
  tag: string;
  value: string;
  matchMode: "exact" | "prefix" | "regex";
  customPattern: string; // The prefix or regex pattern (only used when matchMode is prefix or regex)
}

// Store for the categorization modal
class CategorizationStore {
  isOpen = false;
  isLoading = false;
  isSaving = false;
  error: string | null = null;

  // The event being categorized
  eventIds: string[] = [];
  events: SingleExtractedEventWithRaw[] = [];

  // Form state - now supports multiple tag selections
  selectedTags: TagSelection[] = [];
  targetCategory: string = "";

  // Available categories
  availableCategories = [
    "Productivity",
    "Productivity/Software Development",
    "Productivity/Software Development/IDE",
    "Productivity/Software Development/Research",
    "Productivity/Shell",
    "Productivity/Writing",
    "Communication",
    "Communication/Email",
    "Communication/Chat",
    "Entertainment",
    "Entertainment/Video",
    "Entertainment/Gaming",
    "Entertainment/Social Media",
    "Reference",
    "Reference/Documentation",
    "Social Media",
    "System",
  ];

  constructor() {
    makeAutoObservable(this);
  }

  async open(eventIds: string[]) {
    this.isOpen = true;
    this.isLoading = true;
    this.error = null;
    this.eventIds = eventIds;
    this.selectedTags = [];
    this.targetCategory = "";

    try {
      const events = await api.getSingleEvents(eventIds, false, false);
      runInAction(() => {
        this.events = events;
        this.isLoading = false;
      });
    } catch (e) {
      runInAction(() => {
        this.error = e instanceof Error ? e.message : "Failed to load event";
        this.isLoading = false;
      });
    }
  }

  close() {
    this.isOpen = false;
    this.events = [];
    this.eventIds = [];
    this.selectedTags = [];
  }

  toggleTag(tag: string, value: string) {
    const existingIndex = this.selectedTags.findIndex(
      (t) => t.tag === tag && t.value === value
    );

    if (existingIndex >= 0) {
      this.selectedTags.splice(existingIndex, 1);
    } else {
      this.selectedTags.push({ tag, value, matchMode: "exact", customPattern: value });
    }
  }

  isTagSelected(tag: string, value: string): boolean {
    return this.selectedTags.some((t) => t.tag === tag && t.value === value);
  }

  setTagMatchMode(tag: string, value: string, mode: "exact" | "prefix" | "regex") {
    const selection = this.selectedTags.find((t) => t.tag === tag && t.value === value);
    if (selection) {
      selection.matchMode = mode;
      // Set a sensible default pattern when switching modes
      if (mode === "prefix") {
        // Default to first part before "/" or the whole value
        selection.customPattern = value.includes("/") ? value.split("/")[0] : value;
      } else if (mode === "regex") {
        // Default to escaped value as regex
        selection.customPattern = value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      } else {
        selection.customPattern = value;
      }
    }
  }

  getTagMatchMode(tag: string, value: string): "exact" | "prefix" | "regex" {
    const selection = this.selectedTags.find((t) => t.tag === tag && t.value === value);
    return selection?.matchMode || "exact";
  }

  setTagCustomPattern(tag: string, value: string, pattern: string) {
    const selection = this.selectedTags.find((t) => t.tag === tag && t.value === value);
    if (selection) {
      selection.customPattern = pattern;
    }
  }

  getTagCustomPattern(tag: string, value: string): string {
    const selection = this.selectedTags.find((t) => t.tag === tag && t.value === value);
    return selection?.customPattern || value;
  }

  setTargetCategory(category: string) {
    this.targetCategory = category;
  }

  // Get all unique tags from loaded events
  get availableTags(): Array<{ tag: string; value: string }> {
    const tagMap = new Map<string, Set<string>>();

    for (const event of this.events) {
      // Handle tags that might be a Map or have a .map property
      let tagsObj = event.tags;
      if (tagsObj && typeof tagsObj === "object" && "map" in tagsObj) {
        // Tags might be wrapped in a {map: {...}} structure
        tagsObj = (tagsObj as Record<string, unknown>).map as typeof event.tags;
      }
      if (!tagsObj || typeof tagsObj !== "object") continue;

      for (const [tag, values] of Object.entries(tagsObj)) {
        if (!values) continue;
        // Skip internal/derived tags
        if (tag === "category" || tag === "event-id" || tag === "timetrackrs-raw-id") continue;

        if (!tagMap.has(tag)) {
          tagMap.set(tag, new Set());
        }
        // Handle both array and string values
        const valueArray = Array.isArray(values) ? values : [String(values)];
        for (const value of valueArray) {
          tagMap.get(tag)!.add(value);
        }
      }
    }

    const result: Array<{ tag: string; value: string }> = [];
    for (const [tag, values] of tagMap.entries()) {
      for (const value of values) {
        result.push({ tag, value });
      }
    }

    // Sort by tag name
    return result.sort((a, b) => a.tag.localeCompare(b.tag));
  }

  async save() {
    if (this.selectedTags.length === 0 || !this.targetCategory) {
      this.error = "Please select at least one tag to match and a target category";
      return;
    }

    this.isSaving = true;
    this.error = null;

    try {
      // Fetch existing rule groups
      const groups = await api.getRuleGroups();

      // Find or create a "User Rules" group
      let userGroup = groups.find((g) => g.global_id === "user-custom-rules");

      if (!userGroup) {
        userGroup = {
          global_id: "user-custom-rules",
          data: {
            version: "V1",
            data: {
              name: "User Custom Rules",
              description: "Rules created from the UI",
              editable: true,
              enabled: true,
              rules: [],
            },
          },
        };
        groups.push(userGroup);
      }

      // Create the new rule based on selections
      let newRule;

      if (this.selectedTags.length === 1) {
        // Single tag - use simple rule types
        const sel = this.selectedTags[0];
        if (sel.matchMode === "exact") {
          newRule = {
            enabled: true,
            rule: {
              type: "ExactTagValue" as const,
              tag: sel.tag,
              value: sel.value,
              new_tags: [{ tag: "category", value: this.targetCategory }],
            },
          };
        } else if (sel.matchMode === "prefix") {
          newRule = {
            enabled: true,
            rule: {
              type: "TagValuePrefix" as const,
              tag: sel.tag,
              prefix: sel.customPattern,
              new_tags: [{ tag: "category", value: this.targetCategory }],
            },
          };
        } else {
          // Regex mode - use the custom pattern directly
          newRule = {
            enabled: true,
            rule: {
              type: "TagRegex" as const,
              regexes: [{ tag: sel.tag, regex: sel.customPattern }],
              new_tags: [{ tag: "category", value: this.targetCategory }],
            },
          };
        }
      } else {
        // Multiple tags - use TagRegex with multiple regexes (all must match)
        const regexes = this.selectedTags.map((sel) => {
          let regex: string;
          if (sel.matchMode === "exact") {
            const escaped = sel.value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
            regex = `^${escaped}$`;
          } else if (sel.matchMode === "prefix") {
            // Convert prefix to regex: ^prefix
            const escaped = sel.customPattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
            regex = `^${escaped}`;
          } else {
            // Regex mode - use custom pattern directly
            regex = sel.customPattern;
          }
          return { tag: sel.tag, regex };
        });

        newRule = {
          enabled: true,
          rule: {
            type: "TagRegex" as const,
            regexes,
            new_tags: [{ tag: "category", value: this.targetCategory }],
          },
        };
      }

      // Add the rule to the user group
      userGroup.data.data.rules.push(newRule);

      // Save back to server
      await api.updateRuleGroups(groups);

      // Invalidate extractions for the entire visible timeline range
      // This ensures all events matching the new rule get recategorized
      const rangeStart = timelineStore.rangeStart.getTime();
      const rangeEnd = timelineStore.rangeEnd.getTime();
      await api.invalidateExtractions(rangeStart, rangeEnd);

      runInAction(() => {
        this.isSaving = false;
        this.close();
      });

      // Reload the timeline data (without full page reload)
      await timelineStore.loadData();
    } catch (e) {
      runInAction(() => {
        this.error = e instanceof Error ? e.message : "Failed to save rule";
        this.isSaving = false;
      });
    }
  }
}

export const categorizationStore = new CategorizationStore();

export const CategorizationModal = observer(function CategorizationModal() {
  const store = categorizationStore;
  const [customCategory, setCustomCategory] = useState("");

  if (!store.isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={() => store.close()}
      />

      {/* Modal */}
      <div className="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Categorize Events
          </h2>
          <button
            onClick={() => store.close()}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-4 overflow-y-auto flex-1">
          {store.isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" />
            </div>
          ) : store.error ? (
            <div className="p-4 bg-red-50 dark:bg-red-900/30 rounded-lg text-red-700 dark:text-red-300 text-sm">
              {store.error}
            </div>
          ) : (
            <div className="space-y-6">
              {/* Event preview */}
              <div>
                <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Event Preview
                </h3>
                <div className="p-3 bg-gray-50 dark:bg-gray-900 rounded-lg text-sm">
                  {store.events[0] && (
                    <div className="space-y-1">
                      <p className="text-gray-900 dark:text-white font-medium">
                        {store.events[0].tags["software-window-title"]?.[0] ||
                          store.events[0].tags["software-name"]?.[0] ||
                          "Unknown"}
                      </p>
                      <p className="text-gray-500 dark:text-gray-400 text-xs">
                        {store.events.length} event(s) selected
                      </p>
                    </div>
                  )}
                </div>
              </div>

              {/* Tag selection - multi-select */}
              <div>
                <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Match by Tags
                  <span className="ml-2 text-xs font-normal text-gray-500">
                    (select multiple for AND matching)
                  </span>
                </h3>
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {store.availableTags.map(({ tag, value }) => {
                    const isSelected = store.isTagSelected(tag, value);
                    const matchMode = store.getTagMatchMode(tag, value);

                    return (
                      <div
                        key={`${tag}:${value}`}
                        className={`p-2 rounded-lg transition-colors ${
                          isSelected
                            ? "bg-indigo-50 dark:bg-indigo-900/30 border border-indigo-200 dark:border-indigo-700"
                            : "hover:bg-gray-50 dark:hover:bg-gray-700 border border-transparent"
                        }`}
                      >
                        <label className="flex items-start gap-3 cursor-pointer">
                          <input
                            type="checkbox"
                            checked={isSelected}
                            onChange={() => store.toggleTag(tag, value)}
                            className="mt-1"
                          />
                          <div className="min-w-0 flex-1">
                            <p className="text-xs font-medium text-gray-500 dark:text-gray-400">
                              {tag}
                            </p>
                            <p className="text-sm text-gray-900 dark:text-white break-all">
                              {value}
                            </p>
                          </div>
                        </label>

                        {/* Match mode selector - shown when selected */}
                        {isSelected && (
                          <div className="mt-2 ml-6 space-y-2">
                            <div className="flex gap-1">
                              {(["exact", "prefix", "regex"] as const).map((mode) => (
                                <button
                                  key={mode}
                                  onClick={() => store.setTagMatchMode(tag, value, mode)}
                                  className={`px-2 py-0.5 text-xs rounded transition-colors ${
                                    matchMode === mode
                                      ? "bg-indigo-600 text-white"
                                      : "bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-500"
                                  }`}
                                >
                                  {mode === "exact" ? "Exact" : mode === "prefix" ? "Prefix" : "Regex"}
                                </button>
                              ))}
                            </div>
                            {/* Pattern input for prefix/regex modes */}
                            {(matchMode === "prefix" || matchMode === "regex") && (
                              <div className="flex items-center gap-2">
                                <span className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                                  {matchMode === "prefix" ? "Prefix:" : "Regex:"}
                                </span>
                                <input
                                  type="text"
                                  value={store.getTagCustomPattern(tag, value)}
                                  onChange={(e) => store.setTagCustomPattern(tag, value, e.target.value)}
                                  className="flex-1 px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
                                  placeholder={matchMode === "prefix" ? "Enter prefix..." : "Enter regex pattern..."}
                                />
                              </div>
                            )}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>

                {store.selectedTags.length > 1 && (
                  <p className="mt-2 text-xs text-indigo-600 dark:text-indigo-400">
                    Rule will match when ALL {store.selectedTags.length} selected tags are present
                  </p>
                )}
              </div>

              {/* Category selection */}
              <div>
                <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Assign Category
                </h3>
                <div className="space-y-2">
                  <select
                    value={store.targetCategory}
                    onChange={(e) => store.setTargetCategory(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                  >
                    <option value="">Select a category...</option>
                    {store.availableCategories.map((cat) => (
                      <option key={cat} value={cat}>
                        {cat}
                      </option>
                    ))}
                  </select>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={customCategory}
                      onChange={(e) => setCustomCategory(e.target.value)}
                      placeholder="Or enter custom category..."
                      className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                    />
                    <button
                      onClick={() => {
                        if (customCategory) {
                          store.setTargetCategory(customCategory);
                          setCustomCategory("");
                        }
                      }}
                      disabled={!customCategory}
                      className="px-3 py-2 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg text-sm disabled:opacity-50"
                    >
                      Use
                    </button>
                  </div>
                </div>
              </div>

              {/* Preview */}
              {store.selectedTags.length > 0 && store.targetCategory && (
                <div className="p-3 bg-indigo-50 dark:bg-indigo-900/30 rounded-lg">
                  <p className="text-sm text-indigo-800 dark:text-indigo-200">
                    <span className="font-medium">Rule preview:</span>{" "}
                    {store.selectedTags.length === 1 ? (
                      <>
                        When{" "}
                        <code className="px-1 bg-indigo-100 dark:bg-indigo-800 rounded">
                          {store.selectedTags[0].tag}
                        </code>{" "}
                        {store.selectedTags[0].matchMode === "exact" && "equals"}
                        {store.selectedTags[0].matchMode === "prefix" && "starts with"}
                        {store.selectedTags[0].matchMode === "regex" && "matches regex"}{" "}
                        <code className="px-1 bg-indigo-100 dark:bg-indigo-800 rounded font-mono">
                          {store.selectedTags[0].matchMode === "exact"
                            ? store.selectedTags[0].value
                            : store.selectedTags[0].customPattern}
                        </code>
                      </>
                    ) : (
                      <>
                        When ALL of:{" "}
                        {store.selectedTags.map((sel, i) => (
                          <span key={i}>
                            {i > 0 && " AND "}
                            <code className="px-1 bg-indigo-100 dark:bg-indigo-800 rounded">
                              {sel.tag}
                            </code>
                            {sel.matchMode === "exact" && " = "}
                            {sel.matchMode === "prefix" && " starts with "}
                            {sel.matchMode === "regex" && " matches regex "}
                            <code className="px-1 bg-indigo-100 dark:bg-indigo-800 rounded font-mono">
                              {sel.matchMode === "exact" ? sel.value : sel.customPattern}
                            </code>
                          </span>
                        ))}
                      </>
                    )}
                    , set category to{" "}
                    <code className="px-1 bg-indigo-100 dark:bg-indigo-800 rounded">
                      {store.targetCategory}
                    </code>
                  </p>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-3">
          <button
            onClick={() => store.close()}
            className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => store.save()}
            disabled={store.isSaving || store.selectedTags.length === 0 || !store.targetCategory}
            className="px-4 py-2 text-sm bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {store.isSaving && (
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white" />
            )}
            Create Rule
          </button>
        </div>
      </div>
    </div>
  );
});
