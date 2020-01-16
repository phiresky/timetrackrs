import { observable } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { render } from "react-dom"
import { Activity as _Activity, CapturedData } from "./server"

type Activity = CapturedData & Omit<_Activity, "data">

@observer
class GUI extends React.Component {
	@observable data: "unloaded" | "loading" | { data: Activity[] } = "unloaded"

	constructor(p: {}) {
		super(p)
		this.fetchData()
	}

	async fetchData() {
		this.data = "loading"
		const today = new Date()
		today.setHours(today.getHours() - 3)
		const now = new Date()
		const data = await fetch(
			`http://localhost:8000/fetch-activity/${today.toISOString()}/${now.toISOString()}`,
		)
		this.data = await data.json()
		//console.log(this.data.data)
	}

	render() {
		//const da = groupBy(this.data.data);
		console.log(this.data.data)
		return (
			<div>
				<h1>PC Usage</h1>
				<div>
					{this.data === "unloaded" || this.data === "loading"
						? this.data
						: this.data.data.map(e => {
								const window = e.data.windows.find(
									w => w.window_id === e.data.focused_window,
								)
								return (
									<p key={e.timestamp}>
										{new Date(
											e.timestamp,
										).toLocaleTimeString()}
										:{" "}
										{
											window?.window_properties[
												"_NET_WM_NAME"
											]
										}
									</p>
								)
						  })}
				</div>
			</div>
		)
	}
}

render(<GUI />, document.getElementById("root"))
