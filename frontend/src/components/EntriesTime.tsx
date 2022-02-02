import React from "react"
import { SingleExtractedChunk } from "../server"
import { durationToString, totalDurationSecondsTag } from "../util"
import { timeFmt } from "./Timeline"

export function EntriesTime({
	entries,
}: {
	entries: SingleExtractedChunk[]
}): JSX.Element {
	const duration = totalDurationSecondsTag(entries, "timetrackrs-tracked")
	const from = timeFmt.format(
		new Date(entries[entries.length - 1].from.epochMilliseconds),
	)
	const _to = new Date(entries[0].to_exclusive.epochMilliseconds)
	const to = timeFmt.format(_to)
	const range = from === to ? from : `${from} - ${to}`
	return (
		<>
			{durationToString(duration)} ({range})
		</>
	)
}
