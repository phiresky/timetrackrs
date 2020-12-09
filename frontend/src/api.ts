import { EventData, TagAddReason, TagRuleGroup } from "./server"
export type Activity = {
	id: string
	timestamp: string
	duration: number
	tags: string[]
	tags_reasons: { [tag: string]: TagAddReason }
	raw?: EventData
}

const backend =
	new URLSearchParams(location.search).get("server") ||
	location.protocol + "//" + location.hostname + ":8000/api"

async function handleError(resp: Response): Promise<never> {
	const text = await resp.text()
	console.error(
		"could not fetch data from",
		resp.url.toString(),
		":",
		resp.status,
		text,
	)
	throw Error(
		`could not fetch data from ${resp.url.toString()}: ${
			resp.status
		}: ${text}`,
	)
}
export async function getTimeRange(info: {
	before?: Date
	limit: number
	after?: Date
}): Promise<Activity[]> {
	const url = new URL(backend + "/time-range")
	if (info.before) url.searchParams.set("before", info.before.toISOString())
	if (info.limit) url.searchParams.set("limit", String(info.limit))
	if (info.after) url.searchParams.set("after", info.after.toISOString())
	const resp = await fetch(url.toString())
	if (!resp.ok) {
		return await handleError(resp)
	}
	const { data } = (await resp.json()) as { data: Activity[] }
	return data
}

export async function getSingleEvent(info: { id: string }): Promise<Activity> {
	const url = new URL(backend + "/single-event")
	url.searchParams.set("id", info.id)
	const resp = await fetch(url.toString())
	if (!resp.ok) {
		return await handleError(resp)
	}
	const { data } = (await resp.json()) as { data: Activity }
	return data
}

export async function getTagRules(): Promise<TagRuleGroup[]> {
	const url = new URL(backend + "/rule-groups")
	const resp = await fetch(url.toString())
	if (!resp.ok) {
		return await handleError(resp)
	}
	const { data } = (await resp.json()) as { data: TagRuleGroup[] }
	return data
}

export async function saveTagRules(groups: TagRuleGroup[]): Promise<void> {
	const url = new URL(backend + "/rule-groups")
	const resp = await fetch(url.toString(), {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(groups),
	})
	if (!resp.ok) {
		return await handleError(resp)
	}
	const { data } = (await resp.json()) as { data: void }
	return data
}
