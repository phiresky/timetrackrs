import * as React from "react"
import { useEffect, useState } from "react"
import { getTimeRange } from "../api"
import { useTimeRange } from "./TimeRangeSelector"
import { SingleExtractedChunk } from "../server"
import { ProgressPlugin } from "webpack"
import { observer } from "mobx-react"
import { expectNeverThrow, useMobxEffect } from "../util"
import { observable } from "mobx"
import { Temporal } from "@js-temporal/polyfill"

type TagFilter = string | { tag: string; value?: string; valuePrefix?: string }
type Expression =
	| number
	| TagFilter
	| { minus: [Expression, Expression] }
	| { div: [Expression, Expression] }
type Unit = "duration" | "percentage"
type Props = {
	time: { from: Temporal.ZonedDateTime; to: Temporal.ZonedDateTime }
	calculation: Expression
	fetchFilter?: string
	unit: Unit
}
function isTagFilter(e: Expression): e is TagFilter {
	return typeof e === "string" || (typeof e !== "number" && "tag" in e)
}

function matchTagFilter(
	f: TagFilter,
	[tag, value, _]: [string, string, number],
): boolean {
	if (typeof f === "string") return tag === f
	return (
		tag === f.tag &&
		(!f.value || value === f.value) &&
		(!f.valuePrefix || value.startsWith(f.valuePrefix))
	)
}
function getTagFilterTime(c: SingleExtractedChunk, f: TagFilter): number {
	return c.tags
		.filter((e) => matchTagFilter(f, e))
		.reduce((a, b) => a + b[2], 0)
}
function applyExpression(data: SingleExtractedChunk[], e: Expression): number {
	if (isTagFilter(e)) {
		return data
			.map((ele) => getTagFilterTime(ele, e))
			.reduce((a, b) => a + b, 0)
	} else if (typeof e === "number") {
		return e
	} else if ("div" in e) {
		const [a, b] = e.div
		const ar = applyExpression(data, a)
		const br = applyExpression(data, b)
		console.log("div result", ar, br)
		return ar / br
	} else if ("minus" in e) {
		const [a, b] = e.minus
		return applyExpression(data, a) - applyExpression(data, b)
	}
	throw Error(`unknown expression ${JSON.stringify(e)}`)
}

export const SingleNumberValue: React.FC<Props> = observer((_props) => {
	const [data, setData] = useState(null as null | number)
	const props = observable(_props, observable.struct)
	useMobxEffect(async () => {
		const { calculation } = props
		console.log("calculation", calculation)
		// console.log("props", )
		setData(null)
		const data = await getTimeRange({
			after: props.time.from,
			before: props.time.to,
			tag: props.fetchFilter,
		})
		setData(applyExpression(data, calculation))
	})
	if (data !== null) {
		const value = numberWithUnitToString(data, props.unit)
		return <span>{value}</span>
	}
	return <span>...</span>
})

function numberWithUnitToString(value: number, unit: Unit): string {
	if (unit === "duration") {
		const seconds = ((value / 1000) | 0) % 60
		const minutes = ((value / 1000 / 60) | 0) % 60
		const hours = (value / 1000 / 60 / 60) | 0
		if (hours > 0) {
			return `${hours} h ${minutes} min`
		}
		if (minutes > 0) {
			return `${minutes} min ${seconds} s`
		}
		return `${seconds} s`
	} else if (unit === "percentage") {
		return (value * 100).toFixed(0) + " %"
	}
	expectNeverThrow(unit, "unknown unit")
}
