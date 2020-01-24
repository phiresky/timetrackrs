import _ from "lodash"
import * as PlotlyT from "plotly.js"
import * as PlotlyI from "plotly.js-dist"
import React from "react"
import { agg } from "./ftree"
import { Activity } from "./main"

const Plotly = PlotlyI as typeof PlotlyT

const data: Plotly.Data[] = [
	{
		x: ["giraffes", "orangutans", "monkeys"],
		y: [20, 14, 23],
		type: "histogram",
	},
]

export class Plot extends React.Component<{ data: Activity[] }> {
	r = React.createRef<HTMLDivElement>()
	plot: Plotly.PlotlyHTMLElement | null = null
	async componentDidUpdate() {
		if (this.r.current) {
			// if(this.plot) Plotly.plot
			if (!agg.group) throw Error("a")
			const g = agg.group
			const _gs = _.groupBy(this.props.data, e => {
				const g1 = g(e)
				const g2 = g1.group?.(e)
				if (g2) return g2.key
				return g1.key
			})

			const data: Plotly.Data[] = Object.entries(_gs).map(([key, es]) => {
				return {
					x: es.map(x => new Date(x.timestamp)),
					y: es.map(x => x.duration / 60),
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
