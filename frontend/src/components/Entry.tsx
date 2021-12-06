import { Temporal } from "@js-temporal/polyfill"
import React from "react"
import { routes } from "../routes"
import { SingleExtractedEventWithRaw } from "../server"
import { ModalLink } from "./ModalLink"

export class Entry extends React.Component<SingleExtractedEventWithRaw> {
	render(): React.ReactNode {
		const { id, timestamp_unix_ms } = this.props
		return (
			<span>
				<ModalLink route={routes.singleEvent} args={{ id }} query={{}}>
					Event at{" "}
					{Temporal.Instant.fromEpochMilliseconds(timestamp_unix_ms)
						.toZonedDateTimeISO(Temporal.Now.timeZone())
						.toLocaleString()}
				</ModalLink>
			</span>
		)
	}
}
