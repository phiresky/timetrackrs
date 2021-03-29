import { addSeconds, endOfDay, isThisHour, startOfDay } from "date-fns/esm"
import * as d from "date-fns/esm"
import _ from "lodash"
import { action, computed, makeObservable, observable, toJS } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { ChooserWithChild, CWCRouteMatch } from "./ChooserWithChild"
import { Page } from "./Page"
import { getTag } from "./Timeline"
import Plotly from "react-plotly.js"
import { SingleExtractedEvent } from "../server"
import { differenceInDays, getTime } from "date-fns"
import { Choices, Select } from "./Select"

export function PlotPage(p: { routeMatch: CWCRouteMatch }): React.ReactElement {
	return (
		<Page title="Plot">
			<ChooserWithChild
				routeMatch={p.routeMatch}
				child={Plot}
				containerClass="centerbody"
			/>
		</Page>
	)
}

@observer
export class Plot extends React.Component<{
	events: SingleExtractedEvent[]
	tag: string
}> {
	r = React.createRef<HTMLDivElement>()

	plot: Plotly.PlotlyHTMLElement | null = null

	@observable deep = false
	@computed get binSizes() {
		let totalDur: number
		if (this.aggregators.value.name === "Daily")
			totalDur = 1000 * 60 * 60 * 24
		else if (this.aggregators.value.name === "Weekly")
			totalDur = 1000 * 60 * 60 * 24 * 7
		else {
			totalDur =
				this.dayInfo.last.getTime() - this.dayInfo.first.getTime()
		}
		console.log("total dur: ", totalDur / 1000 / 60 / 60 / 24)
		const allChoices = [
			{ value: 1000 * 60 * 5, name: "5 minutes" },
			{ value: 1000 * 60 * 20, name: "20 minutes" },
			{ value: 1000 * 60 * 60, name: "Hourly" },
			{ value: 1000 * 60 * 60 * 4, name: "4 h" },
			{ value: 1000 * 60 * 60 * 24, name: "Daily" },
		]
		const filteredChoices = allChoices.filter(
			(c) => c.value > totalDur / 1000 && c.value < totalDur / 4,
		)
		if (filteredChoices.length === 0) return observable(Choices(allChoices))
		return observable(Choices(filteredChoices))
	}
	@computed get aggregators() {
		const allChoices = [
			{ mapper: (date: Date) => date, name: "None", visible: true },
			{
				mapper: (date: Date) => {
					const duration = d.intervalToDuration({
						start: d.startOfDay(date),
						end: date,
					})
					const today = d.startOfDay(new Date("2021-01-01"))
					return d.add(today, duration)
				},
				name: "Daily",
				visible: this.dayInfo.days > 2,
			},
			{
				mapper: (date: Date) => {
					const duration = d.intervalToDuration({
						start: d.startOfWeek(date),
						end: date,
					})
					const today = d.startOfWeek(new Date())
					return d.add(today, duration)
				},
				name: "Weekly",
				visible: this.dayInfo.days > 14,
			},
		]
		const filteredChoices = allChoices.filter((c) => c.visible)
		if (filteredChoices.length === 0) return observable(Choices(allChoices))
		return observable(Choices(filteredChoices))
	}

	constructor(p: Plot["props"]) {
		super(p)
		makeObservable(this)
	}
	/*@computed get currentBinSize() {
		const v = this.binSize.value
		if (v.value <= 1000 * 60 && this.dayInfo.days > 1) {
			return this.binSize.choices.find((f) => f.value > 1000 * 60)!
		}
		return v
	}*/

	@computed get dayInfo() {
		return this.getDayInfo(this.props.events)
	}

	getDayInfo(e: { from: Date | number; to_exclusive: Date | number }[]) {
		if (e.length === 0) {
			return {
				firstDay: new Date(),
				lastDay: new Date(),
				days: 0,
				first: new Date(),
				last: new Date(),
				durMs: 0,
			}
		}
		const first = new Date(e[0].from)
		const last = new Date(+e[e.length - 1].to_exclusive - 1)
		const firstDay = startOfDay(first)
		const lastDay = startOfDay(last)
		const days = differenceInDays(lastDay, firstDay) + 1
		const durMs = last.getTime() - first.getTime()
		console.log(first, last)
		return { firstDay, lastDay, days, first, last, durMs }
	}

	private getValue(value: string) {
		if (this.deep) return value
		else return value.split("/")[0]
	}

	@computed get data(): Plotly.Data[] {
		const aggregator = this.aggregators.value
		const binSize = this.binSizes.value.value
		const { days } = this.dayInfo

		type TagValue = string
		type Bucket = string
		type relative_duration = number

		const outData = new Map<TagValue, Map<Bucket, relative_duration>>()

		for (const timechunk of this.props.events) {
			const bucket = aggregator
				.mapper(new Date(timechunk.from - (timechunk.from % binSize)))
				.toJSON()
			for (const [tag, _value, duration] of timechunk.tags) {
				if (tag !== this.props.tag) continue
				const value = this.getValue(_value)
				let tagdata = outData.get(value)
				if (!tagdata) {
					tagdata = new Map()
					outData.set(value, tagdata)
				}
				const bucketdata = tagdata.get(bucket) || 0
				tagdata.set(bucket, bucketdata + duration)
			}
		}
		console.log("time buckets", this.props.events, outData)

		//const { firstDay, lastDay, days: aggDays } = this.getDayInfo(data)

		const data: Plotly.Data[] = [...outData].map(([key, es]) => {
			//

			const aggFactor = 1 // aggDays / days
			const es2 = [...es]

			return {
				/*xaxis: {
					tick0: firstDay,
				},*/
				x: es2.map((x) => x[0]),
				y: es2.map((x) => (x[1] / binSize) * aggFactor),
				type: "bar",
				name: key,
			}
		})
		return data
	}

	render(): React.ReactElement {
		const agg = this.aggregators.value
		return (
			<div>
				Precision:{" "}
				<Select
					target={this.binSizes}
					getName={(e) => e.name}
					getValue={(e) => String(e.value)}
				/>{" "}
				{this.aggregators.choices.length > 1 && (
					<>
						Aggregation:{" "}
						<Select
							target={this.aggregators}
							getName={(e) => e.name}
							getValue={(e) => String(e.name)}
						/>{" "}
					</>
				)}
				<label>
					Deep{" "}
					<input
						type="checkbox"
						checked={this.deep}
						onChange={(e) => (this.deep = e.currentTarget.checked)}
					/>
				</label>
				<Plotly
					className="maxwidth"
					data={this.data}
					layout={{
						autosize: true,
						title: `Time spent by ${this.props.tag}`,
						barmode: "stack",
						legend: { orientation: "h" },

						yaxis: {
							// https://github.com/d3/d3-format/blob/master/README.md#locale_format
							title: "Spent time (%)",
							tickformat: "percent",
							hoverformat: ".0%",
						},
						xaxis: {
							// https://github.com/d3/d3-time-format/blob/master/README.md
							tickformat:
								agg.name == "None"
									? undefined
									: agg.name === "Daily"
									? "%H:%M:%S"
									: agg.name === "Weekly"
									? "%A %H:%M"
									: undefined,
						},
					}}
				/>
			</div>
		)
	}
}
