import React from "react"
import { Switch, Route, Redirect } from "react-router-dom"
import { TagTree } from "./TagTree"
import { Timeline } from "../timeline"
import { SingleEventInfo } from "./SingleEventInfo"

export function Routes() {
	return (
		<Switch>
			<Route path="/timeline">
				<Timeline />
			</Route>
			<Route path="/tag-tree">
				<TagTree />
			</Route>
			<Route exact path="/">
				<Redirect to="/timeline"></Redirect>
			</Route>
			<Route
				path="/single-event/:id"
				render={(p) => (
					<SingleEventInfo
						id={(p.match.params as { id: string }).id}
					></SingleEventInfo>
				)}
			></Route>
			<div>Error 404</div>
		</Switch>
	)
}
