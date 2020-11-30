import { observer, useLocalStore } from "mobx-react"
import React from "react"
import * as dfn from "date-fns"

export function TimeRangeSelectorDefault() {
	return { from: dfn.startOfDay(new Date()), to: new Date() }
}
export const TimeRangeSelector: React.FC<{
	target: { from: Date; to: Date }
}> = observer(({ target }) => {
	const Modes = ["today", "past 7 days", "past month", "custom"] as const
	type Mode = typeof Modes[number]

	const state = useLocalStore(() => ({
		mode: "today" as Mode,
		setMode(mode: Mode) {
			this.mode = mode
			if (mode === "today") {
				target.from = dfn.startOfDay(new Date())
				target.to = new Date()
			}
			if (mode === "past 7 days") {
				target.from = dfn.startOfDay(dfn.subDays(new Date(), 7))
				target.to = new Date()
			}
			if (mode === "past month") {
				target.from = dfn.startOfDay(dfn.subMonths(new Date(), 1))
				target.to = new Date()
			}
		},
	}))

	return (
		<div>
			<select
				value={state.mode}
				onChange={(e) => state.setMode(e.currentTarget.value as Mode)}
			>
				{Modes.map((mode) => (
					<option key={mode} value={mode}>
						{mode}
					</option>
				))}
			</select>
		</div>
	)
})
