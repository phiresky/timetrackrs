import { observable, runInAction } from "mobx"
import { observer } from "mobx-react"
import React, { ReactElement } from "react"
import * as api from "../api"
import { durationToString, totalDuration } from "../util"
import { Activity } from "../api"
import { EntriesTime } from "./EntriesTime"
import { Entry } from "./Entry"
import { Page } from "./Page"
import { TagTree } from "./TagTree"
import { Choices, Select } from "./Select"

export function getTag(tags: string[], tag: string) {
	return tags.find((t) => t.startsWith(tag + ":"))?.slice(tag.length + 1)
}

type Filter = { tagName: string }

interface Grouper {
	name: string
	shouldGroup(a: Activity, b: Activity): boolean
	component: React.ComponentType<{ entries: Activity[]; filter: Filter }>
}
const groupers: Grouper[] = [
	{
		name: "Category",
		shouldGroup({ tags: a }, { tags: b }) {
			return getTag(a, "category") === getTag(b, "category")
		},
		component(p) {
			return (
				<ul>
					<li>
						Category: {getTag(p.entries[0].tags, "category")}
						<TagTree
							events={p.entries}
							tagName={p.filter.tagName}
						/>
					</li>
				</ul>
			)
		},
	},
	{
		name: "Daily",
		shouldGroup(a, b) {
			const d1 = new Date(a.timestamp)
			const d2 = new Date(b.timestamp)
			return (
				d1.toISOString().slice(0, 10) === d2.toISOString().slice(0, 10)
			)
		},
		component(p) {
			return (
				<ul>
					<li>
						Total tracked time:{" "}
						{durationToString(totalDuration(p.entries))}:
						<TagTree
							events={p.entries}
							tagName={p.filter.tagName}
						/>
					</li>
				</ul>
			)
		},
	},
	{
		name: "None",
		shouldGroup(a, b) {
			return true
		},
		component(p) {
			return (
				<ul>
					{p.entries.map((e) => (
						<li key={e.id}>
							<EntriesTime entries={[e]} />
							<Entry {...e} />
						</li>
					))}
				</ul>
			)
		},
	},
]

function group(grouper: Grouper, entries: Activity[]): Activity[][] {
	const res: Activity[][] = []
	let last: Activity | null = null
	let start = 0
	for (const [i, entry] of entries.entries()) {
		if (!last || grouper.shouldGroup(last, entry)) {
			//
		} else {
			res.push(entries.slice(start, i))
			start = i
		}
		last = entry
	}
	if (start < entries.length) res.push(entries.slice(start))
	return res
}

export const timeFmt = new Intl.DateTimeFormat("en-US", {
	hour12: false,
	hour: "numeric",
	minute: "numeric",
})

/*function chooseGrouper(
	entries: Activity[],
	targetCount: number,
	targetOffset: number,
) {
	const bg = groupers.map((g) => {
		const count = group(g, entries).length
		return { g, count }
	})
	bg.sort((a, b) => a.count - b.count)
	//console.log(bg)
	const inx = Math.min(
		bg.length - 1,
		bg.findIndex((e) => e.count >= targetCount) + targetOffset,
	)
	//console.log(inx)
	return bg[inx].g
}*/
function RenderGroup(props: {
	entries: Activity[]
	filter: Filter
	grouper: Grouper
}) {
	const grouper = props.grouper //chooseGrouper(props.entries, 1, 0)
	const C = grouper.component
	const groups = group(grouper, props.entries)
	return (
		<>
			{groups.map((entries) => (
				<section key={entries[0].timestamp}>
					<h4>
						<EntriesTime entries={entries} /> [{grouper.name}]
					</h4>
					<C entries={entries} filter={props.filter} />
				</section>
			))}
		</>
	)
}

export function TimelinePage(): ReactElement {
	return (
		<Page title="Timeline" headerClass="fade-in">
			<Timeline />
		</Page>
	)
}
const detailBy = [
	{ key: "category", name: "Category" },
	{ key: "software-executable-basename", name: "Program" },
]
@observer
export class Timeline extends React.Component {
	@observable data = new Map<string, Activity[]>()
	@observable loading = false
	@observable loadState = "unloaded"
	@observable oldestData = new Date()
	@observable readonly detailBy = Choices(detailBy)
	@observable readonly aggBy = Choices(
		groupers,
		groupers.find((g) => g.name === "Daily"),
	)

	constructor(p: Record<string, unknown>) {
		super(p)
		Object.assign(window, { gui: this })
		void this.fetchData()
	}

	async fetchData() {
		if (this.loading) return
		try {
			this.loading = true
			this.loadState = `loading from ${this.oldestData.toISOString()}`
			const now = new Date()
			const data = await api.getTimeRange({
				before: this.oldestData,
				limit: 1000,
			})
			runInAction(() => {
				let l = null
				for (const d of data) {
					const ts = new Date(d.timestamp)
					const k = ts.toISOString().slice(0, 10)
					l = ts
					let z = this.data.get(k)
					if (!z) {
						z = []
						this.data.set(k, z)
					}
					z.push(d)
				}
				if (l) this.oldestData = l
				this.loadState = "loaded"
			})
		} finally {
			this.loading = false
		}
		//console.log(this.data.data)
	}

	onScroll = (e: React.UIEvent<HTMLDivElement>) => {
		const element = e.currentTarget
		const bottom = element.clientHeight + element.scrollTop
		if (element.scrollHeight - bottom < 300) {
			void this.fetchData()
		}
	}

	render() {
		//const da = groupBy(this.data.data);
		return (
			<div className="timeline">
				<div className="timeline-config">
					<h2>{this.loadState}</h2>
					<div>
						Aggregate by{" "}
						<Select
							target={this.aggBy}
							getValue={(e) => e.name}
							getName={(e) => e.name}
						/>
					</div>
					<div>
						Detail by{" "}
						<Select
							target={this.detailBy}
							getValue={(e) => e.key || "OO"}
							getName={(e) => e.key || "OO"}
						/>
					</div>
				</div>
				<div className="item" onScroll={this.onScroll}>
					<div className="timeline-inner">
						<div>
							{[...this.data.entries()].map(([day, entries]) => {
								return (
									<section className="year" key={day}>
										<h3>{day}</h3>
										<RenderGroup
											entries={entries}
											filter={{
												tagName: this.detailBy.value
													.key,
											}}
											grouper={this.aggBy.value}
										/>
									</section>
								)
							})}
						</div>
					</div>
				</div>
				{/*<Plot data={[...this.data.values()].flat()} />*/}
			</div>
		)
	}
}