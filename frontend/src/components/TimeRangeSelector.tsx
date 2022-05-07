import { observer, useLocalObservable } from "mobx-react"
import React from "react"
import "react-dates/lib/css/_datepicker.css"
import { DateRangePicker, SingleDatePicker } from "react-dates"
import moment from "moment"
import {
	Button,
	ButtonGroup,
	Card,
	DropdownItem,
	DropdownMenu,
	DropdownToggle,
	UncontrolledDropdown,
} from "reactstrap"
import { expectNeverThrow } from "../util"
import { Temporal, toTemporalInstant } from "@js-temporal/polyfill"

const Modes = ["day", "week", "month", "exact"] as const
export type TimeRangeMode = typeof Modes[number]

export type TimeRangeTarget = {
	from: Temporal.ZonedDateTime
	to: Temporal.ZonedDateTime
	mode: TimeRangeMode
}
type TimeRangeStore = {
	focusedW: "startDate" | "endDate" | null
	focused: boolean
	setMode(mode: TimeRangeMode): void
	setDate(
		d: Temporal.ZonedDateTime | undefined,
		end?: Temporal.ZonedDateTime,
	): void
	back(): void
	forward(): void
}

export function useTimeRange(target: TimeRangeTarget): TimeRangeStore {
	const store = useLocalObservable(() => ({
		focusedW: null as "startDate" | "endDate" | null,
		focused: false,
		setMode(mode: TimeRangeMode) {
			target.mode = mode
			if (mode === "day") {
				target.from = target.from.startOfDay()
				target.to = target.from.startOfDay().add({ days: 1 })
			} else if (mode === "week") {
				const s = Temporal.Now.zonedDateTimeISO().subtract({ days: 7 })
				// earlier
				const d =
					Temporal.ZonedDateTime.compare(target.from, s) < 0
						? target.from
						: s
				target.from = d.startOfDay()
				target.to = target.from.add({ weeks: 1 })
			} else if (mode === "month") {
				target.from = target.from.startOfDay().with({ day: 1 })
				target.to = target.from.add({ months: 1 })
			} else if (mode === "exact") {
				// keep
			} else expectNeverThrow(mode)
		},
		setDate(
			d: Temporal.ZonedDateTime | undefined,
			end?: Temporal.ZonedDateTime,
		) {
			console.log("set date", d)
			if (!d) d = Temporal.Now.zonedDateTimeISO()
			target.from = d.startOfDay()
			if (target.mode === "day") target.to = d.add({ days: 1 })
			else if (target.mode === "week") target.to = d.add({ days: 7 })
			else if (target.mode === "month") target.to = d.add({ months: 1 })
			else if (target.mode === "exact") {
				if (!end) throw Error("no end date")
				target.from = d
				target.to = end
			} else expectNeverThrow(target.mode)
		},
		shift(mul: -1 | 1) {
			if (target.mode === "day") {
				this.setDate(target.from.add({ days: mul * 1 }))
			} else if (target.mode === "week") {
				this.setDate(target.from.add({ days: mul * 7 }))
			} else if (target.mode === "month") {
				this.setDate(target.from.add({ months: mul * 1 }))
			} else if (target.mode === "exact") {
				this.setDate(
					target.from.add({ days: mul * 1 }),
					target.to.add({ days: mul * 1 }),
				)
			} else expectNeverThrow(target.mode)
		},
		back() {
			this.shift(-1)
		},
		forward() {
			this.shift(1)
		},
	}))
	return store
}
export const DateTimePicker: React.FC<{
	value: Temporal.PlainDateTime
	onChange: (v: Temporal.PlainDateTime) => void
}> = ({ value, onChange }) => {
	const date = value.toPlainDate().toString()
	const time = value.toPlainTime().toString()
	return (
		<>
			<input
				type="date"
				value={date}
				onChange={(e) =>
					onChange(value.withPlainDate(e.currentTarget.value))
				}
			/>
			<input
				type="time"
				value={time}
				onChange={(e) =>
					onChange(value.withPlainTime(e.currentTarget.value))
				}
			/>
		</>
	)
}
function toZonedDateTime(d: Date): Temporal.ZonedDateTime {
	return toTemporalInstant.call(d).toZonedDateTimeISO(Temporal.Now.timeZone())
}
function legacyMomentToTemporal(m: moment.Moment): Temporal.ZonedDateTime {
	return toZonedDateTime(m.toDate())
}
function temporalToLegacyMoment(t: Temporal.ZonedDateTime): moment.Moment {
	return moment(t.epochMilliseconds)
}
export const TimeRangeSelector: React.FC<{
	target: TimeRangeTarget
}> = observer(({ target }) => {
	const state = useTimeRange(target)
	const commonProps = {
		key: target.mode,
		displayFormat: "YYYY-MM-DD",
		showDefaultInputIcon: true,
	}
	let picker
	if (target.mode === "day")
		picker = (
			<SingleDatePicker
				{...commonProps}
				id="time-range-seli"
				onDateChange={(e) =>
					e && state.setDate(legacyMomentToTemporal(e))
				}
				focused={state.focused}
				onFocusChange={(focused) => (state.focused = focused.focused)}
				numberOfMonths={1}
				date={temporalToLegacyMoment(target.from)}
				isOutsideRange={(d) => d.isAfter(new Date())}
			/>
		)
	else if (target.mode === "week")
		picker = (
			<DateRangePicker
				{...commonProps}
				startDateOffset={(d) => d}
				endDateOffset={(d) => d.add(6, "days")}
				startDateId="time-range-seli1"
				endDateId="timee-range-seli2"
				onDatesChange={(e) =>
					e.startDate &&
					state.setDate(legacyMomentToTemporal(e.startDate))
				}
				focusedInput={state.focusedW}
				onFocusChange={(focused) => (state.focusedW = focused)}
				numberOfMonths={1}
				startDate={temporalToLegacyMoment(target.from)}
				endDate={temporalToLegacyMoment(target.to)}
				isOutsideRange={(d) => d.isAfter(new Date())}
			/>
		)
	else if (target.mode === "month")
		picker = (
			<DateRangePicker
				{...commonProps}
				showDefaultInputIcon
				startDateOffset={(d) => d.startOf("month")}
				endDateOffset={(d) => d.endOf("month")}
				startDateId="time-range-seliq1"
				endDateId="timee-range-seliq2"
				onDatesChange={(e) =>
					e.startDate &&
					state.setDate(legacyMomentToTemporal(e.startDate))
				}
				focusedInput={state.focusedW}
				onFocusChange={(focused) => (state.focusedW = focused)}
				numberOfMonths={1}
				startDate={temporalToLegacyMoment(target.from)}
				endDate={temporalToLegacyMoment(target.to)}
				isOutsideRange={(d) => false}
			/>
		)
	else if (target.mode === "exact") {
		const timeZone = Temporal.Now.timeZone()
		const from = target.from.toPlainDateTime()
		const to = target.to.toPlainDateTime()

		picker = (
			<>
				<DateTimePicker
					value={from}
					onChange={(from) =>
						(target.from = from.toZonedDateTime(timeZone))
					}
				/>
				<DateTimePicker
					value={to}
					onChange={(to) =>
						(target.to = to.toZonedDateTime(timeZone))
					}
				/>
			</>
		)
	} else expectNeverThrow(target.mode)
	return (
		<Card className="time-range-selector mt-3 mb-4">
			<div>
				<Button
					title="day before"
					className="caretbutton"
					onClick={() => state.back()}
				>
					{"<"}
				</Button>
				<select
					className="btn"
					value={target.mode}
					onChange={(e) =>
						state.setMode(e.currentTarget.value as TimeRangeMode)
					}
				>
					{Modes.map((mode) => (
						<option key={mode} value={mode}>
							{mode}
						</option>
					))}
				</select>
				{picker}
				{Temporal.Instant.compare(
					target.to.toInstant(),
					Temporal.Now.instant(),
				) < 0 && (
					<Button
						className="caretbutton"
						title="day after"
						onClick={() => state.forward()}
					>
						{">"}
					</Button>
				)}
			</div>
		</Card>
	)
})

export const TimeRangeSelectorSimple: React.FC<{
	target: TimeRangeTarget
}> = observer(({ target }) => {
	const state = useTimeRange(target)
	let picker: string
	if (target.mode === "day") {
		const today = Temporal.Now.zonedDateTimeISO()
		const date = target.from.toPlainDate().equals(today.toPlainDate())
			? "today"
			: target.from.toPlainDate().equals(today.add({ days: -1 }))
			? "yesterday"
			: target.from.toPlainDate().toString()
		picker = date
	} else if (target.mode === "week") {
		picker =
			target.from.toPlainDate().toString() +
			" to " +
			target.to.toPlainDate().toString()
	} else if (target.mode === "month") {
		picker = target.from.toLocaleString(undefined, {
			month: "short",
			year: "numeric",
		})
	} else {
		throw Error(`unknown mode ${target.mode}`)
	}

	return (
		<ButtonGroup>
			<Button
				title="day before"
				className="caretbutton"
				onClick={() => state.back()}
			>
				{"<"}
			</Button>
			<UncontrolledDropdown className="btn-group">
				<DropdownToggle caret>{picker}</DropdownToggle>
				<DropdownMenu>
					{Modes.map((mode) => (
						<DropdownItem
							key={mode}
							onClick={() => state.setMode(mode)}
						>
							{mode === target.mode ? picker : mode}
						</DropdownItem>
					))}
				</DropdownMenu>
			</UncontrolledDropdown>
			{Temporal.Instant.compare(
				target.to.toInstant(),
				Temporal.Now.instant(),
			) < 0 && (
				<Button
					className="caretbutton"
					title="day after"
					onClick={() => state.forward()}
				>
					{">"}
				</Button>
			)}
		</ButtonGroup>
	)
})
