import _ from "lodash"
import * as React from "react"
import { Activity } from "./main"
import { durationToString, totalDuration } from "./util"

type Filter = {
	name: string | null | undefined
	group?(e: Activity): Filter
}
const Browser: Filter = {
	name: "Browser",
	group(e: Activity) {
		return {
			name: e.data.web_browser?.service,
			group(e: Activity) {
				return { name: e.data.web_browser?.url }
			},
		}
	},
}
const SoftwareDev: Filter = {
	name: "Software Develompent",
	group(e: Activity) {
		return {
			name: e.data.software_development?.project_path,
			group(e: Activity) {
				return {
					name: e.data.software_development?.file_path,
				}
			},
		}
	},
}
const Shell: Filter = {
	name: "Shell",
	group(e: Activity) {
		return { name: e.data.shell?.cwd }
	},
}
const agg: Filter = {
	name: "Activity",
	group(e: Activity) {
		if (e.data.web_browser) return Browser
		if (e.data.software_development) return SoftwareDev
		if (e.data.shell) return Shell

		return { name: "Other" }
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
			{filter.name}: {durString}
		</div>
	) : (
		<></>
	)
	if (!filter.group || !expanded) return <div>{h}</div>
	const g = filter.group
	const _gs = _.groupBy(entries, e => g(e).name)
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
