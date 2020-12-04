import { addMinutes } from "date-fns"
import { addSeconds } from "date-fns/esm"
import _ from "lodash"
import { action } from "mobx"
import * as PlotlyT from "plotly.js"
import * as PlotlyI from "plotly.js-dist"
import React from "react"
import { Activity } from "./api"

const Plotly = PlotlyI as typeof PlotlyT

export class Plot extends React.Component<{ data: Activity[] }> {
	r = React.createRef<HTMLDivElement>()
	plot: Plotly.PlotlyHTMLElement | null = null
	async componentDidUpdate() {
		if (this.r.current) {
			// if(this.plot) Plotly.plot
			if (!categoryAggregate.group) throw Error("a")
			const g = categoryAggregate.group
			const _gs = _.groupBy(this.props.data, (e) => {
				const g1 = g(e)
				const g2 = g1.group?.(e)
				if (g2) return g2.key
				return g1.key
			})

			const data: Plotly.Data[] = Object.entries(_gs).map(([key, es]) => {
				const es2 = es.flatMap((e) => {
					if (e.duration > 600) {
						return Array(Math.ceil(e.duration / 600))
							.fill(0)
							.map(
								(_, i) =>
									({
										...e,
										timestamp: addSeconds(
											new Date(e.timestamp),
											600 * i,
										).toISOString(),
										duration: Math.min(
											600,
											e.duration - 600 * i,
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
			console.log(data)

			this.plot = await Plotly.newPlot(this.r.current, data, {
				barmode: "stack",
				legend: { position: "top", orientation: "h" },
				yaxis: {
					title: "Spent time (minutes)",
					tickformat: "min",
				},
			})
		}
	}
	render() {
		return <div ref={this.r}></div>
	}
}
