import { makeObservable, observable, runInAction } from "mobx"
import { observer } from "mobx-react"
import React, { ReactElement } from "react"
import * as api from "../api"
import { durationToString, totalDurationSeconds } from "../util"
import { EntriesTime } from "./EntriesTime"
import { Entry } from "./Entry"
import { Page } from "./Page"
import { TagTree } from "./TagTree"
import { Choices, Select } from "./Select"
import { SingleExtractedChunk } from "../server"
import { subDays } from "date-fns"
import { Container } from "reactstrap"

export function getTag(
	tags: [string, string, number][],
	tag: string,
	deep = true,
): string | undefined {
	const value = tags.find((t) => t[0] === tag)?.[1]
	if (!deep) {
		return value?.split("/")[0]
	}
	return value
}

type Filter = { tagName: string }

interface Grouper {
	name: string
	shouldGroup(a: SingleExtractedChunk, b: SingleExtractedChunk): boolean
	component: React.ComponentType<{
		entries: SingleExtractedChunk[]
		filter: Filter
	}>
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
							timeChunks={p.entries}
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
			const d1 = new Date(a.from)
			const d2 = new Date(b.from)
			return (
				d1.toISOString().slice(0, 10) === d2.toISOString().slice(0, 10)
			)
		},
		component(p) {
			return (
				<ul>
					<li>
						Total tracked time:{" "}
						{durationToString(totalDurationSeconds(p.entries))}:
						<TagTree
							timeChunks={p.entries}
							tagName={p.filter.tagName}
						/>
					</li>
				</ul>
			)
		},
	},
	/*{
		name: "None",
		shouldGroup(a, b) {
			return true
		},
		component(p) {
			return (
				<ul>
					{p.entries.map((e) => (
						<li key={e.from}>
							<EntriesTime entries={[e]} />
							<Entry {...e} />
						</li>
					))}
				</ul>
			)
		},
	},*/
]

function group(
	grouper: Grouper,
	entries: SingleExtractedChunk[],
): SingleExtractedChunk[][] {
	const res: SingleExtractedChunk[][] = []
	let last: SingleExtractedChunk | null = null
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
	entries: SingleExtractedChunk[]
	filter: Filter
	grouper: Grouper
}) {
	const grouper = props.grouper //chooseGrouper(props.entries, 1, 0)
	const C = grouper.component
	const groups = group(grouper, props.entries)
	return (
		<>
			{groups.map((entries) => (
				<section key={entries[0].from}>
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
		<Page
			title="Timeline"
			headerClass="fade-in"
			containerClass="timeline-container"
		>
			<Container fluid className="bg-gradient-info pt-md-6">
				<Container>
					<Timeline />
				</Container>
			</Container>
		</Page>
	)
}
const detailBy = [
	{ key: "category", name: "Category" },
	{ key: "software-executable-basename", name: "Program" },
	{ key: "project", name: "Project" },
]
@observer
export class Timeline extends React.Component {
	@observable data = new Map<string, SingleExtractedChunk[]>()
	@observable loading = false

	@observable errored = false
	@observable loadState = "unloaded"
	@observable oldestData = new Date()
	@observable gotOldestEver = false
	@observable readonly detailBy = Choices(detailBy)
	@observable readonly aggBy = Choices(
		groupers,
		groupers.find((g) => g.name === "Daily"),
	)
	@observable scrollDiv = React.createRef<HTMLDivElement>()

	constructor(p: Record<string, unknown>) {
		super(p)
		makeObservable(this)
		Object.assign(window, { gui: this })
		void this.fetchData()
	}

	async fetchData(): Promise<void> {
		if (this.loading) return
		try {
			this.loading = true
			this.loadState = `loading from ${this.oldestData.toISOString()}`
			const now = new Date()
			const data = await api.getTimeRange({
				before: this.oldestData,
				after: subDays(this.oldestData, 1),
			})
			data.sort((a, b) => -a.from - b.from)
			runInAction(() => {
				let l = null
				for (const d of data) {
					const ts = new Date(d.from)
					const k = ts.toISOString().slice(0, 10)
					l = ts
					let z = this.data.get(k)
					if (!z) {
						z = []
						this.data.set(k, z)
					}
					z.push(d)
				}
				if (data.length < 10) {
					this.gotOldestEver = true
					console.log(`got oldest!!`, data)
				}
				if (l) this.oldestData = l
				this.loadState = "loaded"
			})
		} catch (e) {
			this.loadState = `error: ${String(e)}`
			this.errored = true
			throw e
		} finally {
			this.loading = false
		}
		//console.log(this.data.data)
	}
	componentDidUpdate(): void {
		setTimeout(() => {
			void this.onScroll()
		}, 0)
	}

	onScroll = async (): Promise<void> => {
		if (this.gotOldestEver || this.errored) return
		const element = this.scrollDiv.current
		if (!element) return
		let i = 0
		while (i++ < 10) {
			const bottom = element.clientHeight + element.scrollTop
			if (element.scrollHeight - bottom < 300) {
				await this.fetchData()
				if (this.gotOldestEver) return
			}
		}
	}

	render(): React.ReactNode {
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
							getName={(e) => e.name || "OO"}
						/>
					</div>
				</div>
				<div
					className="item"
					ref={this.scrollDiv}
					onScroll={this.onScroll}
				>
					<div className="timeline-inner">
						<div>
							{[...this.data.entries()].map(([day, entries]) => {
								return (
									<section className="year" key={day}>
										<h3>{day}</h3>
										<RenderGroup
											entries={entries}
											filter={{
												tagName:
													this.detailBy.value.key,
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
