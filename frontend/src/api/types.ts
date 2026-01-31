// API Types matching the Rust backend

// Backend returns timestamps as objects with $type and unix_timestamp_ms
export interface TimestamptzObject {
  $type: "Instant";
  unix_timestamp_ms: number;
}

// Helper to extract unix ms from timestamp object
export function getUnixMs(ts: TimestamptzObject): number {
  return ts.unix_timestamp_ms;
}

// For input parameters we use plain numbers
export type Timestamptz = number;

export interface ApiResponse<T> {
  data: T;
}

// Tags are a map of tag names to arrays of values
export type Tags = { [key: string]: string[] | undefined };

export interface TagValue {
  tag: string;
  value: string;
}

// Extracted chunk for time range queries
export interface SingleExtractedChunk {
  from: TimestamptzObject;
  to_exclusive: TimestamptzObject;
  tags: [string, string, number][]; // [tag_name, tag_value, count]
}

// Individual event with raw data
export interface SingleExtractedEventWithRaw {
  id: string;
  timestamp_unix_ms: TimestamptzObject;
  duration_ms: number;
  tags: Tags;
  raw: EventData | null;
  tags_reasons: Record<string, TagAddReason> | null;
}

// Event data types (simplified - raw JSON from backend)
export type EventData = unknown;

// Tag add reason
export type TagAddReason =
  | { type: "IntrinsicTag"; raw_data_type: string }
  | { type: "AddedByRule"; matched_tags: TagValue[]; rule: TagRule };

// Tag rules
export type TagRule =
  | { type: "HasTag"; tag: string; new_tags: TagValue[] }
  | { type: "ExactTagValue"; tag: string; value: string; new_tags: TagValue[] }
  | { type: "TagValuePrefix"; tag: string; prefix: string; new_tags: TagValue[] }
  | { type: "TagRegex"; regexes: TagValueRegex[]; new_tags: TagValue[] }
  | { type: "InternalFetcher"; fetcher_id: string }
  | { type: "ExternalFetcher"; fetcher_id: string };

export interface TagValueRegex {
  tag: string;
  regex: string;
}

export interface TagRuleWithMeta {
  enabled: boolean;
  rule: TagRule;
}

export interface TagRuleGroupV1 {
  name: string;
  description: string;
  editable: boolean;
  enabled: boolean;
  rules: TagRuleWithMeta[];
}

export interface TagRuleGroupData {
  version: "V1";
  data: TagRuleGroupV1;
}

export interface TagRuleGroup {
  global_id: string;
  data: TagRuleGroupData;
}

// API Request types
export interface SingleEventsRequest {
  ids: string;
  include_raw: boolean;
  include_reasons: boolean;
}

export interface TimestampSearchRequest {
  backwards: boolean;
  from?: Timestamptz;
}

export interface InvalidateRangeRequest {
  from: Timestamptz;
  to: Timestamptz;
}
