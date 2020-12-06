import { observer } from "mobx-react"
import * as React from "react"

type Choices<T> = { choices: T[]; value: T }
export function Choices<T>(choices: T[], def?: T): Choices<T> {
	return {
		choices,
		value: def || choices[0],
	}
}
function _Select<T>(props: {
	target: Choices<T>
	getValue: (t: T) => string
	getName: (t: T) => string
}): React.ReactElement {
	const { target, getValue, getName } = props
	return (
		<select
			value={getValue(target.value)}
			onChange={(e) =>
				(target.value = target.choices.find(
					(c) => getValue(c) === e.currentTarget.value,
				)!)
			}
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
