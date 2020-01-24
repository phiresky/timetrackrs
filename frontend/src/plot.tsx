//import * as Plotly from "plotly.js-dist"
import React from "react"
import { Activity } from "./main"

const data: Plotly.Data[] = [
	{
		x: ["giraffes", "orangutans", "monkeys"],
		y: [20, 14, 23],
		type: "bar",
	},
]

export class Plot extends React.Component<{ data: Activity[] }> {
	r = React.createRef<HTMLDivElement>()
	componentDidUpdate() {
		/*if (this.r.current) {
			Plotly.newPlot(this.r.current, data)
		}*/
	}
	render() {
		return <div ref={this.r}></div>
	}
}
