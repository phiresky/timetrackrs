import { formatRelative } from "date-fns"
import React from "react"
import { Activity } from "../api"
import { ModalLink } from "./ModalLink"

export class Entry extends React.Component<Activity> {
	render(): React.ReactNode {
		const { id, timestamp } = this.props
		return (
			<span>
				<ModalLink to={`/single-event/${id}`}>
					Event at {formatRelative(new Date(timestamp), new Date())}
				</ModalLink>
			</span>
		)
	}
}
