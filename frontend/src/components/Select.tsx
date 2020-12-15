import { observer } from "mobx-react"
import * as React from "react"

type Choices<T> = { choices: T[]; value: T }
export function Choices<T>(choices: T[], def?: T): Choices<T> {
	if (choices.length === 0) throw Error("no choices")
	return {
		choices,
		value: def || choices[0],
	}
}
function _Select<T>(props: {
	target: Choices<T>
	getValue: (t: T) => string
	getName: (t: T) => string
	onChange?: (t: T) => void
	overrideCurrent?: () => T
}): React.ReactElement {
	const { target, getValue, getName, onChange } = props
	return (
		<select
			value={getValue(
				props.overrideCurrent ? props.overrideCurrent() : target.value,
			)}
			onChange={(e) => {
				const v = target.choices.find(
					(c) => getValue(c) === e.currentTarget.value,
				)
				if (!v) {
					throw Error("select value not found")
				}
				target.value = v
				onChange?.(target.value)
			}}
		>
			{target.choices.map((choice) => (
				<option value={getValue(choice)} key={getValue(choice)}>
					{getName(choice)}
				</option>
			))}
		</select>
	)
}
export const Select = observer(_Select)
