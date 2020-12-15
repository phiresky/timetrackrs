import { addSeconds, endOfDay, isThisHour, startOfDay } from "date-fns/esm"
import * as d from "date-fns/esm"
import _ from "lodash"
import { action, computed, makeObservable, observable, toJS } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { ChooserWithChild } from "./ChooserWithChild"
import { Page } from "./Page"
import { getTag } from "./Timeline"
import Plotly from "react-plotly.js"
import { SingleExtractedEvent } from "../server"
import { differenceInDays, getTime } from "date-fns"
import { Choices, Select } from "./Select"

export function PlotPage(): React.ReactElement {
	return (
		<Page title="Plot">
			<ChooserWithChild
				tag="project"
				child={(p) => <Plot events={p.events} tag="project" />}
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
			{ value: 1000 * 60 * 10, name: "10 minutes" },
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
					const today = d.startOfDay(new Date())
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

	getDayInfo(e: { timestamp: Date | string }[]) {
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
		const first = new Date(e[0].timestamp)
		const last = new Date(e[e.length - 1].timestamp)
		const firstDay = startOfDay(first)
		const lastDay = startOfDay(last)
		const days = differenceInDays(lastDay, firstDay) + 1
		const durMs = last.getTime() - first.getTime()
		return { firstDay, lastDay, days, first, last, durMs }
	}

	@computed get data(): Plotly.Data[] {
		const _gs = _.groupBy(this.props.events, (e) =>
			getTag(e.tags, this.props.tag, this.deep),
		)
		const maxEventSeconds = 300

		const aggregator = this.aggregators.value
		const binSize = this.binSizes.value.value
		const { days } = this.dayInfo

		const data: Plotly.Data[] = Object.entries(_gs).map(([key, es]) => {
			const es2 = es.flatMap((e) => {
				if (e.duration > maxEventSeconds) {
					return Array(Math.ceil(e.duration / maxEventSeconds))
						.fill(0)
						.map((_, i) => ({
							...e,
							timestamp: aggregator.mapper(
								addSeconds(
									new Date(e.timestamp),
									maxEventSeconds * i,
								),
							),
							duration: Math.min(
								maxEventSeconds,
								e.duration - maxEventSeconds * i,
							),
						}))
				} else
					return {
						...e,
						timestamp: aggregator.mapper(new Date(e.timestamp)),
					}
			})
			const { firstDay, lastDay, days: aggDays } = this.getDayInfo(es2)

			const aggFactor = aggDays / days

			return {
				xaxis: {
					tick0: firstDay,
				},
				x: es2.map((x) => x.timestamp),
				y: es2.map((x) => ((x.duration * 1000) / binSize) * aggFactor),
				type: "histogram",
				xbins: {
					start: firstDay.getTime(),
					end: endOfDay(lastDay).getTime(),
					size: binSize,
					//x: days > 7 ? days : days > 1 ? days : 24,
				},
				histfunc: "sum",
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
