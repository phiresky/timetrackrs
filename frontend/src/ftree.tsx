import _ from "lodash"
import * as React from "react"
import {
	Activity,
	KeyedExtractedInfo,
	KeyedOuterUseSpecificSoftware,
} from "./main"
import { durationToString, totalDuration } from "./util"

type Filter = {
	key: string | null | undefined
	name?: string | null | undefined
	group?(e: Activity): Filter
}
const agg: Filter = {
	key: "Activity",
	group(e: Activity) {
		return {
			key: e.data.type,
			group(e) {
				const o: {
					[k in keyof KeyedExtractedInfo]: (
						z: KeyedExtractedInfo[k],
					) => Filter
				} = {
					UseDevice: e => ({
						key: e.specific.type,
						group(e1) {
							const o: {
								[k in keyof KeyedOuterUseSpecificSoftware]: (
									z: KeyedOuterUseSpecificSoftware[k],
								) => Filter
							} = {
								Shell: e => ({
									key: e.specific.cwd,
								}),
								SoftwareDevelopment: e => ({
									key: e.specific.project_path,
									group(e1) {
										return {
											key: e.specific.file_path,
										}
									},
								}),
								WebBrowser: e => ({
									key: e.specific.service,
									group(e1) {
										return { key: e.specific.url }
									},
								}),
								MediaPlayer: e => ({
									key: e.specific.media_name,
								}),
								Unknown: e => ({
									key: "unknown",
								}),
							}
							return o[e.specific.type](e1.data as any)
						},
					}),
					PhysicalActivity: e => ({
						key: "PhysicalActivity",
					}),
				}
				return o[e.data.type](e.data as any)
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
