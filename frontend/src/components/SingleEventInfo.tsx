import { Temporal } from "@js-temporal/polyfill"
import { computed, makeObservable, observable } from "mobx"
import { observer } from "mobx-react"
import { fromPromise, IPromiseBasedObservable } from "mobx-utils"
import * as React from "react"
import { AiOutlineQuestionCircle } from "react-icons/ai"
import * as api from "../api"
import { SingleExtractedEventWithRaw, TagRule } from "../server"
import { intersperse } from "../util"
import { Entry } from "./Entry"

function expectNever<T>(n: never): T {
	return n
}

function formatRelative(from: Temporal.Instant, to: Temporal.Instant) {
	const duration = from.until(to).round({
		smallestUnit: "seconds",
		largestUnit: "years",
		relativeTo: from.toZonedDateTimeISO(Temporal.Now.timeZone()),
	})
	console.log(duration)
	const rtf = new Intl.RelativeTimeFormat("en")
	let ostr = ""
	for (const key of [
		"years",
		"months",
		"weeks",
		"days",
		"hours",
		"minutes",
		"seconds",
	] as const) {
		const val = duration[key]
		if (val !== 0) ostr += " " + rtf.format(val, key)
	}
	return ostr.trim()
}
function reasonstr(rule: TagRule): JSX.Element {
	if (rule.type === "HasTag") return <>has tag</>
	if (rule.type === "ExactTagValue") return <>has tag with exact value</>
	if (rule.type === "InternalFetcher")
		return (
			<>
				InternalFetcher <code>{rule.fetcher_id}</code> converted
			</>
		)
	if (rule.type === "ExternalFetcher")
		return (
			<>
				ExternalFetcher <code>{rule.fetcher_id}</code> converted
			</>
		)
	if (rule.type === "TagValuePrefix")
		return (
			<>
				tag <code>{rule.tag}</code> has prefix {rule.prefix}
			</>
		)
	if (rule.type === "TagRegex")
		return (
			<>
				{intersperse(
					rule.regexes.map((e) => (
						<>
							tag <code>{e.tag}</code> matches regex{" "}
							<code>{e.regex}</code>
						</>
					)),
					() => (
						<> and </>
					),
				)}
				{" so add "}
				{intersperse(
					rule.new_tags.map((t) => (
						<>
							<code>
								{t.tag}:{t.value}
							</code>
						</>
					)),
					() => (
						<> and </>
					),
				)}
			</>
		)
	return <>[{expectNever<TagRule>(rule).type}]</>
}

@observer
export class SingleEventInfoFetch extends React.Component<{ id: string }> {
	constructor(p: { id: string }) {
		super(p)
		makeObservable(this)
	}
	@computed
	get data(): IPromiseBasedObservable<SingleExtractedEventWithRaw | null> {
		return fromPromise(
			api
				.getSingleEvents({
					ids: [this.props.id],
					include_raw: true,
					include_reasons: true,
				})
				.then((e) => e[0]),
		)
	}
	render() {
		if (this.data.state === "pending") return "Loading..."
		if (this.data.state === "rejected") {
			console.log(this.data.value)
			return <div>Could not load: {String(this.data.value)}</div>
		}
		const e = this.data.value
		if (!e) return <div>Event not found</div>
		return <SingleEventInfo event={e} />
	}
}
@observer
export class SingleEventInfo extends React.Component<{
	event: SingleExtractedEventWithRaw
}> {
	constructor(p: SingleEventInfo["props"]) {
		super(p)
		makeObservable(this)
	}
	@observable showReasons = new Set<string>()

	reason(tag: string): JSX.Element {
		const e = this.props.event
		const reason = e.tags_reasons?.[tag]
		if (!reason) return <>[unknown]</>
		return (
			<>
				<br />(
				{reason.type === "IntrinsicTag" ? (
					<>intrinsic tag of data type {e.raw?.data_type})</>
				) : (
					<>
						added because event {reasonstr(reason.rule)}:
						<ul>
							{reason.matched_tags.map((tag) => (
								<li key={tag.tag}>
									{tag.tag}:{tag.value}
									{this.reason(`${tag.tag}:${tag.value}`)}
								</li>
							))}
						</ul>
						{")"}
					</>
				)}
			</>
		)
	}
	render(): React.ReactNode {
		const e = this.props.event
		if (!e) return <div>Event not found</div>
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
					Date:{" "}
					{formatRelative(
						Temporal.Now.instant(),
						e.timestamp_unix_ms,
					)}{" "}
					<small>({e.timestamp_unix_ms.toLocaleString()})</small>
				</p>
				<p>
					Duration:{" "}
					{formatRelative(
						e.timestamp_unix_ms,
						e.timestamp_unix_ms.add(
							Temporal.Duration.from({
								milliseconds: e.duration_ms,
							}),
						),
					)}
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
										) : e.tags_reasons ? (
											<AiOutlineQuestionCircle
												className="clickable"
												onClick={(e) =>
													this.showReasons.add(kv)
												}
											/>
										) : (
											""
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
