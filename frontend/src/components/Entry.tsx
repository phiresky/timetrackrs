import { formatRelative } from "date-fns"
import React from "react"
import { Activity } from "../api"

export class Entry extends React.Component<Activity> {
	render(): React.ReactNode {
		const { timestamp } = this.props
		return (
			<span>Event {formatRelative(new Date(timestamp), new Date())}</span>
		)
	}
}
