import { formatDuration, formatRelative } from "date-fns"
import { computed, observable } from "mobx"
import { observer } from "mobx-react"
import { fromPromise } from "mobx-utils"
import * as React from "react"
import * as api from "../api"
import { Entry } from "./Entry"

@observer
export class SingleEventInfo extends React.Component<{ id: string }> {
	@computed get data() {
		return fromPromise(api.getSingleEvent({ id: this.props.id }))
	}
	render(): React.ReactNode {
		if (this.data.state === "pending") return "Loading..."
		if (this.data.state === "rejected") {
			console.log(this.data.value)
			return <div>Could not load: {String(this.data.value)}</div>
		}
		const e = this.data.value
		console.log("raw data", e)
		return (
			<div>
				<h1>
					<Entry {...e} />
				</h1>
				<p>
					Unique ID: <code>{e.id}</code>
				</p>
				<p>
					Date: {new Date(e.timestamp).toLocaleString()} (
					{formatRelative(new Date(e.timestamp), new Date())})
				</p>
				<p>Duration: {formatDuration({ seconds: e.duration })}</p>
				<div>
					Tags:{" "}
					<ul>
						{e.data.tags.map((tag) => (
							<li key={tag}>{tag}</li>
						))}
					</ul>
				</div>
				{e.raw && (
					<div>
						<div>
							Extracted Data:{" "}
							<pre className="raw-json">
								{JSON.stringify(e.data, null, 2)}
							</pre>
						</div>
					</div>
				)}
				{e.raw && (
					<div>
						<div>Source: {e.raw.data_type}</div>
						<div>
							Raw Data:{" "}
							<pre className="raw-json">
								{JSON.stringify(e.raw.data, null, 2)}
							</pre>
						</div>
					</div>
				)}
			</div>
		)
	}
}
