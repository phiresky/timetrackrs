import { EnrichedExtractedInfo } from "./server"
export type Activity = {
	id: string
	timestamp: string
	duration: number
	data: EnrichedExtractedInfo
}

export async function getTimeRange(info: {
	before?: Date
	limit: number
	after?: Date
}): Promise<Activity[]> {
	const backend =
		new URLSearchParams(location.search).get("server") ||
		location.protocol + "//" + location.hostname + ":8000"
	const url = new URL(backend + "/fetch-info")
	if (info.before) url.searchParams.set("before", info.before.toISOString())
	if (info.limit) url.searchParams.set("limit", String(info.limit))
	if (info.after) url.searchParams.set("after", info.after.toISOString())
	const resp = await fetch(url.toString())
	if (!resp.ok) {
		console.error(
			"could not fetch data from",
			url.toString(),
			":",
			resp.status,
			await resp.text(),
		)
	}
	const { data } = (await resp.json()) as { data: Activity[] }
	return data
}
