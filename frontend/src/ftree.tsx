import _ from "lodash"
import * as React from "react"
import { Activity } from "./main"
import { durationToString, totalDuration } from "./util"

type Filter = {
	key: string | null | undefined
	name?: string | null | undefined
	group?(e: Activity): Filter
}
const Browser: Filter = {
	key: "Browser",
	group(e: Activity) {
		return {
			key: e.data.web_browser?.service,
			group(e: Activity) {
				return { key: e.data.web_browser?.url }
			},
		}
	},
}
const SoftwareDev: Filter = {
	key: "Software Develompent",
	group(e: Activity) {
		return {
			key: e.data.software_development?.project_path,
			group(e: Activity) {
				return {
					key: e.data.software_development?.file_path,
				}
			},
		}
	},
}
const Shell: Filter = {
	key: "Shell",
	group(e: Activity) {
		return { key: e.data.shell?.cwd }
	},
}
const agg: Filter = {
	key: "Activity",
	group(e: Activity) {
		if (e.data.web_browser) return Browser
		if (e.data.software_development) return SoftwareDev
		if (e.data.shell) return Shell

		return {
			key: "Other",
			group(e) {
				return {
					key: e.data.software?.identifier,
					name:
						e.data.software?.unique_name ||
						e.data.software?.identifier,
				}
			},
		}
	},
}

export function SummaryFilter(p: {
	entries: Activity[]
	filter?: Filter
	header?: boolean
}) {
	const { entries, filter = agg, header = true } = p
	const [expanded, setExpanded] = React.useState(!header)
	const durString = durationToString(totalDuration(p.entries))
	const h = header ? (
		<div className="clickable" onClick={e => setExpanded(!expanded)}>
			{filter.name || filter.key}: {durString}
		</div>
	) : (
		<></>
	)
	if (!filter.group || !expanded) return <div>{h}</div>
	const g = filter.group
	const _gs = _.groupBy(entries, e => g(e).key)
	const gs = Object.entries(_gs).sort((a, b) => b[1].length - a[1].length)
	return (
		<div>
			{h}
			<ul>
				{gs.map(([name, entries]) => (
					<li key={name}>
						<SummaryFilter
							entries={entries}
							filter={g(entries[0])}
						/>
					</li>
				))}
			</ul>
		</div>
	)
}
