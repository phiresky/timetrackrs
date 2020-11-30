import React from "react"
import { durationToString, totalDuration } from "../util"
import { Activity } from "../api"
import { timeFmt } from "./Timeline"

export function EntriesTime({ entries }: { entries: Activity[] }) {
	const duration = totalDuration(entries)
	const from = timeFmt.format(new Date(entries[entries.length - 1].timestamp))
	const _to = new Date(entries[0].timestamp)
	_to.setSeconds(_to.getSeconds() + entries[0].duration)
	const to = timeFmt.format(_to)
	const range = from === to ? from : `${from} - ${to}`
	return (
		<>
			{durationToString(duration)} ({range})
		</>
	)
}
