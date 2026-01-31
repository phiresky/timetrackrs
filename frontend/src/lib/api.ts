import { Temporal } from '@js-temporal/polyfill'
import type { ApiTypesTS, ProgressReport, TagRuleGroup } from '../server'

type ApiTypes = { [T in ApiTypesTS['type']]: ApiTypesTS & { type: T } }

type ApiRequest<T extends keyof ApiTypes> = 'request' extends keyof ApiTypes[T]
  ? ApiTypes[T]['request']
  : never
type ApiResponse<T extends keyof ApiTypes> =
  'response' extends keyof ApiTypes[T] ? ApiTypes[T]['response'] : never

type ApiResponseO<T> = { data: T }

const backend =
  new URLSearchParams(location.search).get('server') ||
  new URL('/api', location.href).toString()

export function progressEvents(
  subscriber: (event: ProgressReport[]) => void
): EventSource {
  const eventSource = new EventSource(backend + '/progress-events')
  eventSource.addEventListener('message', (event) => {
    subscriber(JSON.parse(event.data as string) as ProgressReport[])
  })
  return eventSource
}

async function handleError(resp: Response): Promise<never> {
  const text = await resp.text()
  let data: { message: string } | null = null
  try {
    data = JSON.parse(text) as { message: string }
  } catch {
    //
  }
  if (data && data.message) {
    throw Error(`Error from server: ${data.message}`)
  }
  console.error(
    'could not fetch data from',
    resp.url.toString(),
    ':',
    resp.status,
    text
  )
  throw Error(
    `could not fetch data from ${resp.url.toString()}: ${resp.status}: ${text}`
  )
}

type KnownTypes =
  | { $type: 'Instant'; unix_timestamp_ms: number }
  | { $type: undefined }

function deserialize(_key: string, value: unknown) {
  if (typeof value === 'object' && value !== null) {
    const valueObj = value as KnownTypes
    if (
      '$type' in valueObj &&
      valueObj.$type &&
      typeof valueObj.$type === 'string'
    ) {
      if (valueObj.$type === 'Instant') {
        return Temporal.Instant.fromEpochMilliseconds(
          valueObj.unix_timestamp_ms
        )
      }
    }
  }
  return value
}

function removeNulls(_key: string, value: unknown) {
  if (value === null) return undefined
  return value
}

async function doApiRequest<N extends keyof ApiTypes>(
  path: N,
  info: ApiRequest<N>,
  options: { method: 'GET' | 'POST' } = { method: 'GET' }
): Promise<ApiResponse<N>> {
  const params = new URLSearchParams(
    JSON.parse(JSON.stringify(info, removeNulls)) as Record<string, string>
  ).toString()
  const url = new URL(`${backend}/${path.replace(/_/g, '-')}?${params}`)
  const resp = await fetch(url.toString(), options)
  if (!resp.ok) {
    return await handleError(resp)
  }
  const responseText = await resp.text()
  const { data } = JSON.parse(responseText, deserialize) as ApiResponseO<
    ApiResponse<N>
  >
  return data
}

export async function timestampSearch(
  info: ApiRequest<'timestamp_search'>
): Promise<ApiResponse<'timestamp_search'>> {
  return doApiRequest('timestamp_search', info)
}

export async function getTimeRange(
  info: ApiRequest<'time_range'>
): Promise<ApiResponse<'time_range'>> {
  return doApiRequest('time_range', info)
}

export async function invalidateExtractions(
  info: ApiRequest<'invalidate_extractions'>
): Promise<ApiResponse<'invalidate_extractions'>> {
  return doApiRequest('invalidate_extractions', info, { method: 'POST' })
}

export async function getKnownTags(): Promise<ApiResponse<'get_known_tags'>> {
  return doApiRequest('get_known_tags', [])
}

export async function getSingleEvents(
  info: ApiRequest<'single_events'>
): Promise<ApiResponse<'single_events'>> {
  return doApiRequest('single_events', info)
}

export async function getTagRules(): Promise<
  ApiTypes['rule_groups']['response']
> {
  return doApiRequest('rule_groups', [])
}

export async function saveTagRules(groups: TagRuleGroup[]): Promise<void> {
  const url = new URL(backend + '/rule-groups')
  const resp = await fetch(url.toString(), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(groups),
  })
  if (!resp.ok) {
    return await handleError(resp)
  }
  const { data } = (await resp.json()) as { data: void }
  return data
}
