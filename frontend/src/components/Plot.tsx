import { addSeconds, endOfDay, isThisHour, startOfDay } from "date-fns/esm"
import _ from "lodash"
import { action, computed, makeObservable, observable, toJS } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { ChooserWithChild } from "./ChooserWithChild"
import { Page } from "./Page"
import { getTag } from "./Timeline"
import Plotly from "react-plotly.js"
import { SingleExtractedEvent } from "../server"
import { differenceInDays } from "date-fns"
import { Choices, Select } from "./Select"

export function PlotPage(): React.ReactElement {
	return (
		<Page title="Plot">
			<ChooserWithChild child={Plot} containerClass="centerbody" />
		</Page>
	)
}

@observer
export class Plot extends React.Component<{ events: SingleExtractedEvent[] }> {
	r = React.createRef<HTMLDivElement>()

	plot: Plotly.PlotlyHTMLElement | null = null

	@observable deep = false
	@computed get binSizes() {
		const totalDur =
			this.dayInfo.last.getTime() - this.dayInfo.first.getTime()
		const allChoices = [
			{ value: 1000 * 60, name: "Minute" },
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
		if (this.props.events.length === 0) {
			return {
				firstDay: new Date(),
				lastDay: new Date(),
				days: 0,
				first: new Date(),
				last: new Date(),
			}
		}
		const first = new Date(this.props.events[0].timestamp)
		const last = new Date(
			this.props.events[this.props.events.length - 1].timestamp,
		)
		const firstDay = startOfDay(first)
		const lastDay = startOfDay(last)
		const days = differenceInDays(lastDay, firstDay) + 1
		return { firstDay, lastDay, days, first, last }
	}

	@computed get data(): Plotly.Data[] {
		const _gs = _.groupBy(this.props.events, (e) =>
			getTag(e.tags, "category", this.deep),
		)
		const maxEventSeconds = 300
		const data: Plotly.Data[] = Object.entries(_gs).map(([key, es]) => {
			const es2 = es.flatMap((e) => {
				if (e.duration > maxEventSeconds) {
					return Array(Math.ceil(e.duration / maxEventSeconds))
						.fill(0)
						.map(
							(_, i) =>
								({
									...e,
									timestamp: addSeconds(
										new Date(e.timestamp),
										maxEventSeconds * i,
									).toISOString(),
									duration: Math.min(
										maxEventSeconds,
										e.duration - maxEventSeconds * i,
									),
								} as SingleExtractedEvent),
						)
				} else return e
			})
			const { firstDay, lastDay } = this.dayInfo
			return {
				xaxis: {
					tick0: firstDay,
				},
				x: es2.map((x) => new Date(x.timestamp)),
				y: es2.map(
					(x) => (x.duration / this.binSizes.value.value) * 1000,
				),
				type: "histogram",
				xbins: {
					start: firstDay.getTime(),
					end: endOfDay(lastDay).getTime(),
					size: this.binSizes.value.value,
					//x: days > 7 ? days : days > 1 ? days : 24,
				},
				histfunc: "sum",
				name: key,
			}
		})
		return data
	}

	render(): React.ReactElement {
		return (
			<div>
				Precision:{" "}
				<Select
					target={this.binSizes}
					getName={(e) => e.name}
					getValue={(e) => String(e.value)}
				/>{" "}
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
						title: "Plot by Category",
						barmode: "stack",
						legend: { orientation: "h" },

						yaxis: {
							title: "Spent time (%)",
							tickformat: "percent",
						},
					}}
				/>
			</div>
		)
	}
}
