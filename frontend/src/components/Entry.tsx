import { formatRelative } from "date-fns"
import React from "react"
import { routes } from "../routes"
import { SingleExtractedEvent } from "../server"
import { ModalLink } from "./ModalLink"

export class Entry extends React.Component<SingleExtractedEvent> {
	render(): React.ReactNode {
		const { id, timestamp } = this.props
		return (
			<span>
				<ModalLink route={routes.singleEvent} args={{ id }} query={{}}>
					Event at {formatRelative(new Date(timestamp), new Date())}
				</ModalLink>
			</span>
		)
	}
}
