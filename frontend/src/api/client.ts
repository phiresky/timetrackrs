import type {
  ApiResponse,
  SingleExtractedChunk,
  SingleExtractedEventWithRaw,
  TagRuleGroup,
  Timestamptz,
} from "./types";

const API_BASE = "/api";

async function fetchApi<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }

  const result: ApiResponse<T> = await response.json();
  return result.data;
}

// Convert unix ms timestamp to ISO string for the backend
function toIsoString(timestamp: Timestamptz): string {
  return new Date(timestamp).toISOString();
}

function toQueryString(params: object): string {
  const searchParams = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      searchParams.append(key, String(value));
    }
  }
  return searchParams.toString();
}

export const api = {
  async getTimeRange(
    after: Timestamptz,
    before: Timestamptz,
    tag?: string
  ): Promise<SingleExtractedChunk[]> {
    const query = toQueryString({
      after: toIsoString(after),
      before: toIsoString(before),
      tag,
    });
    return fetchApi<SingleExtractedChunk[]>(`/time-range?${query}`);
  },

  async getSingleEvents(
    ids: string[],
    includeRaw = false,
    includeReasons = false
  ): Promise<SingleExtractedEventWithRaw[]> {
    const query = toQueryString({
      ids: ids.join(","),
      include_raw: includeRaw,
      include_reasons: includeReasons,
    });
    return fetchApi<SingleExtractedEventWithRaw[]>(`/single-events?${query}`);
  },

  async getRuleGroups(): Promise<TagRuleGroup[]> {
    return fetchApi<TagRuleGroup[]>("/rule-groups");
  },

  async updateRuleGroups(groups: TagRuleGroup[]): Promise<void> {
    await fetchApi<void>("/rule-groups", {
      method: "POST",
      body: JSON.stringify(groups),
    });
  },

  async getKnownTags(): Promise<string[]> {
    return fetchApi<string[]>("/get-known-tags");
  },

  async searchTimestamp(
    backwards: boolean,
    from?: Timestamptz
  ): Promise<Timestamptz | null> {
    const query = toQueryString({
      backwards,
      from: from !== undefined ? toIsoString(from) : undefined,
    });
    return fetchApi<Timestamptz | null>(`/timestamp-search?${query}`);
  },

  async invalidateExtractions(from: Timestamptz, to: Timestamptz): Promise<void> {
    const query = toQueryString({
      from: toIsoString(from),
      to: toIsoString(to),
    });
    await fetchApi<void>(`/invalidate-extractions?${query}`, {
      method: "POST",
    });
  },
};
