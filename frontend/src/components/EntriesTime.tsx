import React from "react"
import { SingleExtractedEvent } from "../server"
import { durationToString, totalDurationSeconds } from "../util"
import { timeFmt } from "./Timeline"

export function EntriesTime({
	entries,
}: {
	entries: SingleExtractedEvent[]
}): JSX.Element {
	const duration = totalDurationSeconds(entries)
	const from = timeFmt.format(
		new Date(entries[entries.length - 1].timestamp_unix_ms),
	)
	const _to = new Date(entries[0].timestamp_unix_ms)
	_to.setSeconds(_to.getSeconds() + entries[0].duration_ms)
	const to = timeFmt.format(_to)
	const range = from === to ? from : `${from} - ${to}`
	return (
		<>
			{durationToString(duration)} ({range})
		</>
	)
}
