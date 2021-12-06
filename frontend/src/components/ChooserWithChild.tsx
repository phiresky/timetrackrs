import { Temporal } from "@js-temporal/polyfill"
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
type A = {
	child: React.ComponentType<{
		timeChunks: SingleExtractedChunk[]
		tag: undefined
	}>
	chooseTag: false
}
type B = {
	child: React.ComponentType<{
		timeChunks: SingleExtractedChunk[]
		tag: string
	}>
	chooseTag: true
}
export const ChooserWithChild: React.FC<
	{
		containerClass?: string
		routeMatch: CWCRouteMatch
	} & (A | B)
> = observer((p) => {
	const store = useLocalObservable(() => ({
		timeRange: {
			from: Temporal.ZonedDateTime.from(
				p.routeMatch.queryArgs.from ||
					Temporal.Now.zonedDateTimeISO().startOfDay(),
			),
			to: Temporal.ZonedDateTime.from(
				p.routeMatch.queryArgs.to ||
					Temporal.Now.zonedDateTimeISO()
						.add({ days: 1 })
						.startOfDay(),
			),
			mode: (p.routeMatch.queryArgs.range || "day") as TimeRangeMode,
		},
		get data(): IPromiseBasedObservable<SingleExtractedChunk[]> {
			const params = {
				after: this.timeRange.from,
				before: this.timeRange.to,
				tag: p.chooseTag ? this.tag.value : undefined,
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
			from: store.timeRange.from.toString(),
			to: store.timeRange.to.toString(),
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
			Time Range: <TimeRangeSelector target={store.timeRange} />
			{p.chooseTag && (
				<>
					{" "}
					Tag:{" "}
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
				</>
			)}
			<div>
				{store.data.case({
					fulfilled: (v) => (
						<>
							{React.createElement(p.child as any, {
								timeChunks: v,
								...(p.chooseTag
									? { tag: store.tag.value }
									: {}),
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
