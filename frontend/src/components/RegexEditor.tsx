import { observer } from "mobx-react"
import * as React from "react"
import AutosizeInput from "./AutosizeInput"

export const RegexEditor: React.FC<{
	value: string
	editable: boolean
	onChange: (regex: string) => void
}> = observer(({ value, editable, onChange }) => {
	if (!value.startsWith("^")) {
		onChange("^" + value)
		return <>...</>
	}
	if (!value.endsWith("$")) {
		onChange(value + "$")
		return <>...</>
	}
	const inner = value.slice(1, -1)
	return (
		<span className="regex-editor input-border">
			^
			<AutosizeInput
				minWidth={30}
				value={inner}
				disabled={!editable}
				onChange={
					editable
						? (e: React.ChangeEvent<HTMLInputElement>) =>
								onChange("^" + e.currentTarget.value + "$")
						: undefined
				}
			/>
			$
		</span>
	)
})
