import { observable, runInAction } from "mobx"
import { observer } from "mobx-react"
import React from "react"
import { render } from "react-dom"
import { aggregates as detailers, Filter, SummaryFilter } from "./ftree"
import { Plot } from "./plot"
import { EnrichedExtractedInfo, ExtractedInfo } from "./server"
import "./style.scss"
import { Timeline } from "./timeline"
import { durationToString, totalDuration } from "./util"
import {
	BrowserRouter as Router,
	Switch,
	Route,
	Link,
	Redirect,
} from "react-router-dom"
import { TagTree } from "./tag-tree"

function Main() {
	return (
		<Router>
			<Switch>
				<Route path="/timeline">
					<Timeline />
				</Route>
				<Route path="/tag-tree">
					<TagTree />
				</Route>
				<Redirect to="/timeline"></Redirect>
			</Switch>
		</Router>
	)
}
render(<Main />, document.getElementById("root"))
