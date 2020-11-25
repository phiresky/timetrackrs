import { observable, runInAction } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { render } from "react-dom"
import { aggregates as detailers, Filter, SummaryFilter } from "./ftree"
import { Plot } from "./plot"
import { EnrichedExtractedInfo, ExtractedInfo } from "./server"
import "./style.scss"
import { durationToString, totalDuration } from "./util"

export type Activity = {
	id: string
	timestamp: string
	duration: number
	data: EnrichedExtractedInfo
}
type Keyed<
	T extends { [k in discriminator]: string | number | symbol },
	discriminator extends keyof T
> = {
	[k in T[discriminator]]: Omit<
		Extract<T, { [z in discriminator]: k }>,
		discriminator
	>
}
export type KeyedExtractedInfo = Keyed<ExtractedInfo, "type">

type _UseSoftware<T> = T extends { type: "InteractWithDevice" } ? T : never
export type UseSoftware = _UseSoftware<ExtractedInfo>

type KeyedUseSpecificSoftware = Keyed<UseSoftware["specific"], "type">

export type KeyedOuterUseSpecificSoftware = {
	[k in keyof KeyedUseSpecificSoftware]: UseSoftware & {
		specific: KeyedUseSpecificSoftware[k]
	}
}

/*type KeyedOuter<
	TDiscriminator extends string,
	TInner extends string,
	T extends { [TKey in TDiscriminator]: Record<TInner, string> }
> = {
	[TKey in T[TDiscriminator][TInner]]: T extends Record<
		TDiscriminator,
		Record<TInner, TKey>
	>
		? Omit<T, TDiscriminator>
		: never
}

type KeyedUseSpecificSoftware = KeyedOuter<"specific", "type", UseSoftware>*/

type KeyedReactComp<T> = { [k in keyof T]: React.ComponentType<T[k]> }

const useSpecificSoftwareComponents: KeyedReactComp<KeyedOuterUseSpecificSoftware> = {
	Shell(e) {
		return <div>Shell in {e.specific.cwd}</div>
	},
	WebBrowser(e) {
		return <div>Browser at {e.specific.service}</div>
	},
	SoftwareDevelopment(e) {
		return <div>Software Development of {e.specific.project_path}</div>
	},
	MediaPlayer(e) {
		return <div>Consumed Media: {e.specific.media_name}</div>
	},
	DeviceStateChange(e) {
		return (
			<div>
				{e.specific.change} device {e.general.hostname}
			</div>
		)
	},
	Unknown(e) {
		return (
			<div>
				Used {e.general.device_type}: {e.general.title}
			</div>
		)
	},
}

/*const softwareComponents: {k in keyof }*/
const entryComponents: KeyedReactComp<KeyedExtractedInfo> = {
	PhysicalActivity(e) {
		return <div>*dance*</div>
	},
	InteractWithDevice(e) {
		const Comp = useSpecificSoftwareComponents[e.specific.type]
		return <Comp {...(e as any)} />
	},
}

interface Grouper {
	name: string
	shouldGroup(a: Activity, b: Activity): boolean
	component: React.ComponentType<{ entries: Activity[]; filter: Filter }>
}
const groupers: Grouper[] = [
	/*{
		name: "specificComputerProgram",
		shouldGroup({ data: a }, { data: b }) {
			if(a.type === "InteractWithDevice" && b.type === "InteractWithDevice") {
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
	},*/
	{
		name: "UsedComputer",
		shouldGroup(a, b) {
			const d1 = new Date(a.timestamp)
			const d2 = new Date(b.timestamp)
			const distanceSeconds = Math.abs(d1.getTime() - d2.getTime()) / 1000
			if (distanceSeconds > 2 * (a.duration + b.duration)) return false
			return a.data.info.type === "InteractWithDevice" &&
				b.data.info.type === "InteractWithDevice"
				? a.data.info.general.hostname === b.data.info.general.hostname
				: false
		},
		component(p) {
			const type =
				p.entries[0].data.info.type === "InteractWithDevice"
					? p.entries[0].data.info.general?.device_type || "UNK"
					: "UNK"

			return (
				<ul>
					<li>
						Used {type} for{" "}
						{durationToString(totalDuration(p.entries))}: By{" "}
						<SummaryFilter
							entries={p.entries}
							header={false}
							filter={p.filter}
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
						{durationToString(totalDuration(p.entries))}: By{" "}
						<SummaryFilter
							entries={p.entries}
							header={false}
							filter={p.filter}
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

class Entry extends React.Component<Activity> {
	render() {
		const { data } = this.props
		const E = entryComponents[data.info.type] as any
		console.log(E)
		return <E {...data.info} />
		//return "unk: " + data.software?.title
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
	const _to = new Date(entries[0].timestamp)
	_to.setSeconds(_to.getSeconds() + entries[0].duration)
	const to = timeFmt.format(_to)
	const range = from === to ? from : `${from} - ${to}`
	return (
		<>
			{durationToString(duration)} ({range})
		</>
	)
}

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

function Choices<T>(choices: T[], def?: T) {
	return {
		choices,
		value: def || choices[0],
	}
}
function Select<T>(props: {
	target: { choices: T[]; value: T }
	getValue: (t: T) => string
	getName: (t: T) => string
}) {
	const { target, getValue, getName } = props
	return (
		<select
			value={getValue(target.value)}
			onChange={(e) =>
				(target.value = target.choices.find(
					(c) => getValue(c) === e.currentTarget.value,
				)!)
			}
		>
			{target.choices.map((choice) => (
				<option value={getValue(choice)} key={getValue(choice)}>
					{getName(choice)}
				</option>
			))}
		</select>
	)
}

@observer
class GUI extends React.Component {
	@observable data = new Map<string, Activity[]>()
	@observable loading = false
	@observable loadState = "unloaded"
	@observable oldestData = new Date().toISOString()
	@observable readonly detailBy = Choices(detailers)
	@observable readonly aggBy = Choices(groupers)

	constructor(p: Record<string, unknown>) {
		super(p)
		Object.assign(window, { gui: this })
		this.fetchData()
	}

	async fetchData() {
		if (this.loading) return
		try {
			this.loading = true
			this.loadState = `loading from ${this.oldestData}`
			const now = new Date()
			const backend =
				new URLSearchParams(location.search).get("server") ||
				location.protocol + "//" + location.hostname + ":8000"
			const url = new URL(backend + "/fetch-info")
			// url.searchParams.set("from", today.toISOString())
			url.searchParams.set("before", this.oldestData)
			url.searchParams.set("limit", "1000")
			const resp = await fetch(url.toString())
			if (!resp.ok) {
				console.error(
					"could not fetch data from",
					url.toString(),
					":",
					resp.status,
					await resp.text(),
				)
			}
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
					<div id="timeline">
						<div>
							{[...this.data.entries()].map(([day, entries]) => {
								return (
									<section className="year" key={day}>
										<h3>{day}</h3>
										<RenderGroup
											entries={entries}
											filter={this.detailBy.value}
											grouper={this.aggBy.value}
										/>
									</section>
								)
							})}
						</div>
					</div>
				</div>
				<Plot data={[...this.data.values()].flat()} />
			</div>
		)
	}
}

render(<GUI />, document.getElementById("root"))
