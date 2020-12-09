import { formatDuration, formatRelative } from "date-fns"
import { computed, makeObservable, observable } from "mobx"
import { observer } from "mobx-react"
import { fromPromise } from "mobx-utils"
import * as React from "react"
import { AiOutlineQuestionCircle } from "react-icons/ai"
import * as api from "../api"
import { Entry } from "./Entry"

@observer
export class SingleEventInfo extends React.Component<{ id: string }> {
	constructor(p: { id: string }) {
		super(p)
		makeObservable(this)
	}
	@computed get data() {
		return fromPromise(api.getSingleEvent({ id: this.props.id }))
	}
	@observable showReasons = new Set<string>()

	reason(tag: string) {
		if (this.data.state !== "fulfilled") return <>wat</>
		const e = this.data.value
		const reason = e.tags_reasons[tag]
		return (
			<>
				<br />(
				{reason.type === "IntrinsicTag" ? (
					<>intrinsic tag)</>
				) : (
					<>
						added because{" "}
						{reason.rule.type === "TagRegex"
							? reason.rule.regexes.join(",")
							: reason.rule.fetcher_id}{" "}
						matches tag{" "}
						<ul>
							{reason.matched_tags.map((tag) => (
								<li key={tag}>
									{tag}
									{this.reason(tag)}
								</li>
							))}
						</ul>
						)
					</>
				)}
			</>
		)
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
					Date: {formatRelative(new Date(e.timestamp), new Date())}{" "}
					<small>({new Date(e.timestamp).toLocaleString()})</small>
				</p>
				<p>
					Duration:{" "}
					{formatDuration({
						seconds: e.duration % 60,
						minutes: ((e.duration / 60) | 0) % 60,
						hours: (e.duration / 60 / 60) | 0,
					})}
				</p>
				<div>
					Tags:
					<ul>
						{e.tags.map((tag) => (
							<li key={tag}>
								{tag}
								{this.showReasons.has(tag) ? (
									this.reason(tag)
								) : (
									<AiOutlineQuestionCircle
										onClick={(e) =>
											this.showReasons.add(tag)
										}
									/>
								)}
							</li>
						))}
					</ul>
				</div>
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
