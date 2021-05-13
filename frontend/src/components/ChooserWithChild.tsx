import * as dfn from "date-fns"
import { observer, useLocalObservable } from "mobx-react"
import { fromPromise, IPromiseBasedObservable } from "mobx-utils"
import React from "react"
import Select from "react-select"
import * as api from "../api"
import { SingleExtractedChunk } from "../server"
import { useMobxEffect } from "../util"
import {
	TimeRangeMode,
	TimeRangeSelector,
	TimeRangeTarget,
} from "./TimeRangeSelector"

type QA = Partial<{ tag: string; from: string; to: string; range: string }>
export type CWCRouteMatch = {
	queryArgs: QA
	replace(route: undefined, args: undefined, queryArgs: QA): void
}
export const ChooserWithChild: React.FC<{
	child: React.ComponentType<{ events: SingleExtractedChunk[]; tag: string }>

	containerClass?: string
	routeMatch: CWCRouteMatch
}> = observer((p) => {
	const store = useLocalObservable(() => ({
		timeRange: {
			from: dfn.startOfDay(
				new Date(p.routeMatch.queryArgs.from || new Date()),
			),
			to: dfn.endOfDay(new Date(p.routeMatch.queryArgs.to || new Date())),
			mode: (p.routeMatch.queryArgs.range || "day") as TimeRangeMode,
		},
		get data(): IPromiseBasedObservable<SingleExtractedChunk[]> {
			const params = {
				after: this.timeRange.from,
				before: this.timeRange.to,
				tag: this.tag.value,
				limit: 100000,
			}
			return fromPromise(
				api.getTimeRange(params).then((data) => {
					data.sort((a, b) => a.from - b.from)
					console.log(data)
					return data
				}),
			)
		},
		get tags(): IPromiseBasedObservable<
			{ value: string; label: string }[]
		> {
			return fromPromise(
				api
					.getKnownTags()
					.then((e) => e.map((e) => ({ value: e, label: e }))),
			)
		},
		tag: {
			value: p.routeMatch.queryArgs.tag || "category",
			label: p.routeMatch.queryArgs.tag || "category",
		},
	}))
	useMobxEffect(() => {
		const oldArgs = p.routeMatch.queryArgs
		const newArgs = {
			tag: store.tag.value,
			from: dfn.format(store.timeRange.from, "yyyy-MM-dd"),
			to: dfn.format(store.timeRange.to, "yyyy-MM-dd"),
			range: store.timeRange.mode,
		}
		if (
			newArgs.tag !== oldArgs.tag ||
			newArgs.from !== oldArgs.from ||
			newArgs.to !== oldArgs.to ||
			newArgs.range !== oldArgs.range
		) {
			console.log("new args!", newArgs, "oldArgs", oldArgs)
			p.routeMatch.replace(undefined, undefined, newArgs)
		}
	})

	return (
		<div className={`container ${p.containerClass || ""}`}>
			Time Range: <TimeRangeSelector target={store.timeRange} /> Tag:{" "}
			<span style={{ display: "inline-block", width: 200 }}>
				{store.tags.case({
					fulfilled: (v) => (
						<Select
							options={v}
							value={store.tag}
							onChange={(e) => {
								console.log("ch", e)
								store.tag = e || {
									value: "category",
									label: "category",
								}
							}}
						/>
					),
					pending: () => <>...</>,
					rejected: () => <>Could not connect to server</>,
				})}
			</span>
			<div>
				{store.data.case({
					fulfilled: (v) => (
						<>
							{React.createElement(p.child, {
								events: v,
								tag: store.tag.value,
							})}
							<small>
								found {v.length.toString()} events between{" "}
								{store.timeRange.from.toLocaleString()} to{" "}
								{store.timeRange.to.toLocaleString()}
							</small>
						</>
					),
					pending: () => <>Loading events...</>,
					rejected: (e) => {
						console.error("o", e)
						return <>{String(e)}</>
					},
				})}
			</div>
		</div>
	)
})

export const LoadEvents: React.FC<{
	child: React.ComponentType<{ events: SingleExtractedChunk[]; tag: string }>

	containerClass?: string
	timeRange: TimeRangeTarget
	tag: string
}> = observer((p) => {
	const store = useLocalObservable(() => ({
		get data(): IPromiseBasedObservable<SingleExtractedChunk[]> {
			const params = {
				after: p.timeRange.from,
				before: p.timeRange.to,
				tag: p.tag,
				limit: 100000,
			}
			return fromPromise(
				api.getTimeRange(params).then((data) => {
					data.sort((a, b) => a.from - b.from)
					console.log(data)
					return data
				}),
			)
		},
	}))

	return (
		<div className={`container ${p.containerClass || ""}`}>
			{store.data.case({
				fulfilled: (v) => (
					<>
						{React.createElement(p.child, {
							events: v,
							tag: p.tag,
						})}
					</>
				),
				pending: () => <>Loading events...</>,
				rejected: (e) => {
					console.error("o", e)
					return <>{String(e)}</>
				},
			})}
		</div>
	)
})
