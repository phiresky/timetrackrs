import {
	observable,
	autorun,
	computed,
	makeAutoObservable,
	makeObservable,
} from "mobx"
import { observer } from "mobx-react"
import { IPromiseBasedObservable, fromPromise } from "mobx-utils"
import React from "react"
import {
	TimeRangeSelector,
	TimeRangeSelectorDefault,
} from "./TimeRangeSelector"
import * as dfn from "date-fns"
import * as api from "../api"

@observer
export class ChooserWithChild extends React.Component<{
	child: React.ComponentType<{ events: api.Activity[] }>
	containerClass?: string
}> {
	@observable
	timeRange = TimeRangeSelectorDefault()
	constructor(p: ChooserWithChild["props"]) {
		super(p)
		makeObservable(this)
		autorun(() => console.log("from", this.timeRange.from))
	}

	@computed get data(): IPromiseBasedObservable<api.Activity[]> {
		const params = {
			after: this.timeRange.from,
			before: this.timeRange.to,
			limit: 100000,
		}
		return fromPromise(api.getTimeRange(params))
	}

	render(): React.ReactNode {
		console.log(this.data.value)
		return (
			<div className={`container ${this.props.containerClass || ""}`}>
				Time Range: <TimeRangeSelector target={this.timeRange} />
				<div>
					{this.data.case({
						fulfilled: (v) => (
							<>
								{React.createElement(this.props.child, {
									events: v,
								})}
								<small>
									found {v.length.toString()} events between{" "}
									{this.timeRange.from.toLocaleString()} to{" "}
									{this.timeRange.to.toLocaleString()}
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
	}
}
