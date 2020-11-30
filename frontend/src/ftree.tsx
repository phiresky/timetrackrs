import _ from "lodash"
import * as React from "react"
import {
	Activity,
	KeyedExtractedInfo,
	KeyedOuterUseSpecificSoftware,
} from "./main"
import { durationToString, totalDuration } from "./util"

export type Filter = {
	key: string | null | undefined
	name?: string | null | undefined
	group?: (e: Activity) => Filter
}
export const categoryAggregate: Filter = {
	key: "Activity",
	group(e: Activity) {
		return {
			key: e.data.info.type,
			group(e) {
				const o: {
					[k in keyof KeyedExtractedInfo]: (
						z: KeyedExtractedInfo[k],
					) => Filter
				} = {
					InteractWithDevice: (e) => ({
						key: e.specific.type,
						group(e1) {
							const o: {
								[k in keyof KeyedOuterUseSpecificSoftware]: (
									z: KeyedOuterUseSpecificSoftware[k],
								) => Filter
							} = {
								Shell: (e) => ({
									key: e.specific.cwd,
								}),
								SoftwareDevelopment: (e) => ({
									key: e.specific.project_path,
									group(e1) {
										return {
											key: e.specific.file_path,
										}
									},
								}),
								WebBrowser: (e) => ({
									key: e.specific.service,
									group(e1) {
										return { key: e.specific.url }
									},
								}),
								MediaPlayer: (e) => ({
									key: e.specific.media_name,
								}),
								DeviceStateChange: (e) => ({
									key: e.specific.change,
								}),
								Unknown: (e) => ({
									key: e.general.identifier,
								}),
							}
							return o[e.specific.type](e1.data.info as any)
						},
					}),
					PhysicalActivity: (e) => ({
						key: "PhysicalActivity",
					}),
				}
				return o[e.data.info.type](e.data.info as any)
			},
		}
	},
}

function Parec(e: Activity, position: number): Filter {
	const path = (e.data.uri || "Unknown").replace(/\/+/, "/").slice(position)
	const inx = path.indexOf("/")
	if (inx === -1)
		return {
			key: path,
		}
	else {
		return {
			key: path.slice(0, inx),
			group: (e) => Parec(e, position + inx + 1),
		}
	}
}

export const pathRecursiveFilter: Filter = {
	key: "Path",
	group(e: Activity) {
		return Parec(e, 0)
	},
}
export const aggregates = [categoryAggregate, pathRecursiveFilter]

export function SummaryFilter(p: {
	entries: Activity[]
	filter?: Filter
	header?: boolean
	initialExpanded?: boolean
}): JSX.Element {
	const {
		entries,
		filter = categoryAggregate,
		header = true,
		initialExpanded = false,
	} = p
	const [expanded, setExpanded] = React.useState(!header || initialExpanded)
	const durString = durationToString(totalDuration(p.entries))
	const h = header ? (
		<div className="clickable" onClick={(e) => setExpanded(!expanded)}>
			{durString} {filter.name || filter.key}
		</div>
	) : (
		<></>
	)
	if (!filter.group || !expanded) return <div>{h}</div>
	const g = filter.group
	const _gs = _.groupBy(entries, (e) => g(e).key)
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
							initialExpanded={initialExpanded}
						/>
					</li>
				))}
			</ul>
		</div>
	)
}
