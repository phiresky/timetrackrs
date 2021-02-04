import { formatDuration, formatRelative } from "date-fns"
import { computed, makeObservable, observable } from "mobx"
import { observer } from "mobx-react"
import { fromPromise } from "mobx-utils"
import * as React from "react"
import { AiOutlineQuestionCircle } from "react-icons/ai"
import * as api from "../api"
import { TagRule } from "../server"
import { Entry } from "./Entry"

function expectNever<T>(n: never): T {
	return n
}
function reasonstr(rule: TagRule) {
	if (rule.type === "HasTag") return "has"
	if (rule.type === "ExactTagValue") return "has tag with exact value"
	if (rule.type === "InternalFetcher")
		return `InternalFetcher ${rule.fetcher_id} converted`
	if (rule.type === "ExternalFetcher")
		return `ExternalFetcher ${rule.fetcher_id} converted`
	if (rule.type === "TagValuePrefix")
		return `tag ${rule.tag} has prefix ${rule.prefix}`
	if (rule.type === "TagRegex")
		return (
			rule.regexes
				.map((e) => `tag ${e.tag} matches regex ${e.regex}`)
				.join(" and ") +
			` so add ${rule.new_tags
				.map((t) => `${t.tag}:${t.value}`)
				.join(" and ")}`
		)
	return expectNever<TagRule>(rule).type
}
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
		if (!e) return <>Event not found: {this.props.id}</>
		const reason = e.tags_reasons[tag]
		if (!reason) return <>[unknown]</>
		return (
			<>
				<br />(
				{reason.type === "IntrinsicTag" ? (
					<>intrinsic tag)</>
				) : (
					<>
						added because {reasonstr(reason.rule)} tag{" "}
						<ul>
							{reason.matched_tags.map((tag) => (
								<li key={tag.tag}>
									{tag.tag}:{tag.value}
									{this.reason(`${tag.tag}:${tag.value}`)}
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
		if (!e) return <div>Event not found</div>
		console.log("raw data", e)
		const duration = e.duration_ms / 1000
		return (
			<div>
				<h1>
					<Entry {...e} />
				</h1>
				<p>
					Unique ID: <code>{e.id}</code>
				</p>
				<p>
					Date:{" "}
					{formatRelative(new Date(e.timestamp_unix_ms), new Date())}{" "}
					<small>
						({new Date(e.timestamp_unix_ms).toLocaleString()})
					</small>
				</p>
				<p>
					Duration:{" "}
					{formatDuration({
						seconds: duration % 60,
						minutes: ((duration / 60) | 0) % 60,
						hours: (duration / 60 / 60) | 0,
					})}
				</p>
				<div>
					Tags:
					<ul>
						{Object.entries(e.tags.map).map(([key, values]) =>
							values?.map((value) => {
								const kv = `${key}:${value}`
								return (
									<li key={kv}>
										{key}: {value}{" "}
										{this.showReasons.has(kv) ? (
											this.reason(kv)
										) : (
											<AiOutlineQuestionCircle
												onClick={(e) =>
													this.showReasons.add(kv)
												}
											/>
										)}
									</li>
								)
							}),
						)}
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
