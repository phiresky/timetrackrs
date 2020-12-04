import React from "react"

export function Choices<T>(choices: T[], def?: T) {
	return {
		choices,
		value: def || choices[0],
	}
}
export function Select<T>(props: {
	target: { choices: T[]; value: T }
	getValue: (t: T) => string
	getName: (t: T) => string
}) {
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
