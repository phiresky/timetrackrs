import React from "react"
import { SingleExtractedChunk } from "../server"
import { durationToString, totalDurationSeconds } from "../util"
import { timeFmt } from "./Timeline"

export function EntriesTime({
	entries,
}: {
	entries: SingleExtractedChunk[]
}): JSX.Element {
	const duration = totalDurationSeconds(entries)
	const from = timeFmt.format(new Date(entries[entries.length - 1].from))
	const _to = new Date(entries[0].to_exclusive)
	const to = timeFmt.format(_to)
	const range = from === to ? from : `${from} - ${to}`
	return (
		<>
			{durationToString(duration)} ({range})
		</>
	)
}
