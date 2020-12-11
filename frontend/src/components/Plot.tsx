import { addSeconds } from "date-fns/esm"
import _ from "lodash"
import { action, computed, makeObservable, observable } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { Activity } from "../api"
import { ChooserWithChild } from "./ChooserWithChild"
import { Page } from "./Page"
import { getTag } from "./Timeline"
import Plotly from "react-plotly.js"

export function PlotPage(): React.ReactElement {
	return (
		<Page title="Plot">
			<ChooserWithChild child={Plot} containerClass="centerbody" />
		</Page>
	)
}

@observer
export class Plot extends React.Component<{ events: Activity[] }> {
	r = React.createRef<HTMLDivElement>()

	plot: Plotly.PlotlyHTMLElement | null = null

	@observable deep = false

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
								} as Activity),
						)
				} else return e
			})
			return {
				x: es2.map((x) => new Date(x.timestamp)),
				y: es2.map((x) => x.duration / 60),
				type: "histogram",
				nbinsx: 100,
				histfunc: "sum",
				name: key,
			}
		})
		return data
	}

	constructor(p: Plot["props"]) {
		super(p)
		makeObservable(this)
	}
	render(): React.ReactElement {
		return (
			<div>
				<label>
					<input
						type="checkbox"
						checked={this.deep}
						onChange={(e) => (this.deep = e.currentTarget.checked)}
					/>{" "}
					Deep
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
							title: "Spent time (minutes)",
							tickformat: "min",
						},
					}}
				/>
			</div>
		)
	}
}
