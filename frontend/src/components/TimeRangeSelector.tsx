import { observer, useLocalObservable } from "mobx-react"
import React from "react"
import * as dfn from "date-fns"
import "react-dates/initialize"
import "react-dates/lib/css/_datepicker.css"
import {
	DateRangePicker,
	DayPickerRangeController,
	SingleDatePicker,
} from "react-dates"
import moment from "moment"

export function TimeRangeSelectorDefault() {
	return { from: dfn.startOfDay(new Date()), to: new Date() }
}
export const TimeRangeSelector: React.FC<{
	target: { from: Date; to: Date }
}> = observer(({ target }) => {
	const Modes = ["day", "week", "month"] as const
	type Mode = typeof Modes[number]

	const state = useLocalObservable(() => ({
		mode: "day" as Mode,
		focusedW: null as "startDate" | "endDate" | null,
		focused: false,
		setMode(mode: Mode) {
			this.mode = mode
			if (mode === "day") {
				target.from = dfn.startOfDay(target.from)
				target.to = dfn.endOfDay(target.from)
			}
			if (mode === "week") {
				const s = dfn.subDays(new Date(), 7)
				const d = dfn.min([target.from, s])
				target.from = dfn.startOfDay(d)
				target.to = dfn.endOfDay(dfn.addDays(d, 6))
			}
			if (mode === "month") {
				target.from = dfn.startOfMonth(target.from)
				target.to = dfn.endOfMonth(target.from)
			}
		},
		setDate(d: Date | undefined) {
			console.log("set date", d)
			if (!d) d = new Date()
			target.from = dfn.startOfDay(d)
			if (this.mode === "day") target.to = dfn.endOfDay(d)
			else if (this.mode === "week")
				target.to = dfn.endOfDay(dfn.addDays(d, 6))
			else if (this.mode === "month") target.to = dfn.endOfMonth(d)
		},
	}))
	const commonProps = {
		key: state.mode,
		displayFormat: "YYYY-MM-DD",
		showDefaultInputIcon: true,
	}
	let picker
	if (state.mode === "day")
		picker = (
			<SingleDatePicker
				{...commonProps}
				id="time-range-seli"
				onDateChange={(e) => state.setDate(e?.toDate())}
				focused={state.focused}
				onFocusChange={(focused) => (state.focused = focused.focused)}
				numberOfMonths={1}
				date={moment(target.from)}
				isOutsideRange={(d) => d.isAfter(new Date())}
			/>
		)
	if (state.mode === "week")
		picker = (
			<DateRangePicker
				{...commonProps}
				startDateOffset={(d) => d}
				endDateOffset={(d) => d.add(6, "days")}
				startDateId="time-range-seli1"
				endDateId="timee-range-seli2"
				onDatesChange={(e) => state.setDate(e.startDate?.toDate())}
				focusedInput={state.focusedW}
				onFocusChange={(focused) => (state.focusedW = focused)}
				numberOfMonths={1}
				startDate={moment(target.from)}
				endDate={moment(target.to)}
				isOutsideRange={(d) => d.isAfter(new Date())}
			/>
		)
	if (state.mode === "month")
		picker = (
			<DateRangePicker
				{...commonProps}
				showDefaultInputIcon
				startDateOffset={(d) => d.startOf("month")}
				endDateOffset={(d) => d.endOf("month")}
				startDateId="time-range-seliq1"
				endDateId="timee-range-seliq2"
				onDatesChange={(e) => state.setDate(e.startDate?.toDate())}
				focusedInput={state.focusedW}
				onFocusChange={(focused) => (state.focusedW = focused)}
				numberOfMonths={1}
				startDate={moment(target.from)}
				endDate={moment(target.to)}
				isOutsideRange={(d) => false}
			/>
		)
	return (
		<div>
			<button
				title="day before"
				className="caretbutton"
				onClick={() => state.setDate(dfn.subDays(target.from, 1))}
			>
				{"<"}
			</button>
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
			{picker}
			{target.to < new Date() && (
				<button
					className="caretbutton"
					title="day after"
					onClick={() => state.setDate(dfn.addDays(target.from, 1))}
				>
					{">"}
				</button>
			)}
		</div>
	)
})
