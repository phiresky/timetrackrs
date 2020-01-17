import { observable, runInAction } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { render } from "react-dom"
import { SummaryFilter } from "./ftree"
import { ExtractedInfo } from "./server"
import { durationToString, totalDuration } from "./util"

export type Activity = {
	id: string
	timestamp: string
	duration: number
	data: ExtractedInfo
}

const entryComponents: {
	[k in keyof ExtractedInfo]?: (
		e: { [k2 in keyof Omit<ExtractedInfo, k>]: ExtractedInfo[k2] } &
			{ [ki in k]: NonNullable<ExtractedInfo[ki]> },
	) => React.ReactNode
} = {
	shell(e) {
		return <div>Shell in {e.shell.cwd}</div>
	},
	web_browser(e) {
		return <div>Browser at {e.web_browser.service} </div>
	},
	software_development(e) {
		return (
			<div>
				Software Development of {e.software_development.project_path}
			</div>
		)
	},
	software(e) {
		return (
			<div>
				Used {e.software.device_type}: {e.software.title}
			</div>
		)
	},
}

interface Grouper {
	name: string
	shouldGroup(a: Activity, b: Activity): boolean
	component: React.ComponentType<{ entries: Activity[] }>
}
const groupers: Grouper[] = [
	{
		name: "specificComputerProgram",
		shouldGroup({ data: a }, { data: b }) {
			if (a.shell && a.shell.cwd === b.shell?.cwd) return true
			if (
				a.software_development &&
				a.software_development.project_path ===
					b.software_development?.project_path
			)
				return true
			if (
				a.web_browser &&
				a.web_browser.service === b.web_browser?.service
			)
				return true
			return false
		},
		component(p) {
			return (
				<ul>
					<li>
						<Entry {...p.entries[0]} />
					</li>
				</ul>
			)
		},
	},
	{
		name: "UsedComputer",
		shouldGroup(a, b) {
			const d1 = new Date(a.timestamp)
			const d2 = new Date(b.timestamp)
			const distanceSeconds = Math.abs(d1.getTime() - d2.getTime()) / 1000
			if (distanceSeconds > 2 * (a.duration + b.duration)) return false
			return a.data.software
				? a.data.software.hostname === b.data.software?.hostname
				: false
		},
		component(p) {
			const type = p.entries[0].data.software?.device_type || "UNK"

			return (
				<ul>
					<li>
						Used {type} for{" "}
						{durationToString(totalDuration(p.entries))}:
						<SummaryFilter entries={p.entries} header={false} />
					</li>
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

class Entry extends React.Component<Activity> {
	render() {
		const { data } = this.props
		for (const k of Object.keys(data) as (keyof typeof data)[]) {
			const E = entryComponents[k] as any
			if (data[k] && E) return <E {...data} />
		}
		return "unk: " + data.software?.title
	}
}

const timeFmt = new Intl.DateTimeFormat("en-US", {
	hour12: false,
	hour: "numeric",
	minute: "numeric",
})

function EntriesTime({ entries }: { entries: Activity[] }) {
	const duration = totalDuration(entries)
	const from = timeFmt.format(new Date(entries[entries.length - 1].timestamp))
	const to = timeFmt.format(new Date(entries[0].timestamp))
	const range = from === to ? from : `${from} - ${to}`
	return (
		<>
			{durationToString(duration)} ({range})
		</>
	)
}

function chooseGroup(
	entries: Activity[],
	targetCount: number,
	targetOffset: number,
) {
	const bg = groupers.map(g => {
		const count = group(g, entries).length
		return { g, count }
	})
	bg.sort((a, b) => a.count - b.count)
	console.log(bg)
	const inx = Math.min(
		bg.length - 1,
		bg.findIndex(e => e.count >= targetCount) + targetOffset,
	)
	console.log(inx)
	return bg[inx].g
}
function RenderGroup(props: { entries: Activity[] }) {
	const grouper = chooseGroup(props.entries, 1, 0)
	const C = grouper.component
	const groups = group(grouper, props.entries)
	return (
		<>
			{groups.map(entries => (
				<section key={entries[0].timestamp}>
					<h4>
						<EntriesTime entries={entries} /> [{grouper.name}]
					</h4>
					<C entries={entries} />
				</section>
			))}
		</>
	)
}

@observer
class GUI extends React.Component {
	@observable data = new Map<string, Activity[]>()
	@observable loading = false
	@observable loadState = "unloaded"
	@observable oldestData = new Date().toISOString()
	constructor(p: {}) {
		super(p)
		this.fetchData()
	}

	async fetchData() {
		if (this.loading) return
		try {
			this.loading = true
			this.loadState = `loading from ${this.oldestData}`
			const now = new Date()
			const url = new URL(
				location.protocol +
					"//" +
					location.hostname +
					":8000/fetch-info",
			)
			// url.searchParams.set("from", today.toISOString())
			url.searchParams.set("before", this.oldestData)
			url.searchParams.set("limit", "300")
			const resp = await fetch(url.toString())
			const { data }: { data: Activity[] } = await resp.json()
			runInAction(() => {
				let l = null
				for (const d of data) {
					const ts = new Date(d.timestamp).toISOString()
					const k = ts.slice(0, 10)
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
			this.fetchData()
		}
	}

	render() {
		//const da = groupBy(this.data.data);
		return (
			<div className="container">
				<div className="header">
					<h1>Personal Timeline</h1>
					<h2>{this.loadState}</h2>
				</div>
				<div className="item" onScroll={this.onScroll}>
					<div id="timeline">
						<div>
							{[...this.data.entries()].map(([day, entries]) => {
								return (
									<section className="year" key={day}>
										<h3>{day}</h3>
										<RenderGroup entries={entries} />
									</section>
								)
							})}
						</div>
					</div>
				</div>
			</div>
		)
	}
}

render(<GUI />, document.getElementById("root"))
