import React from "react"
import { Switch, Route, Redirect, RouteComponentProps } from "react-router-dom"
import { TagTree, TagTreePage } from "./TagTree"
import { Timeline, TimelinePage } from "./Timeline"
import { SingleEventInfo } from "./SingleEventInfo"
import { ChooserWithChild } from "./ChooserWithChild"
import { CategoryChart } from "./CategoryChart"
import { PlotPage } from "./Plot"

export function Routes(): React.ReactElement {
	return (
		<Switch>
			<Route path="/plot">
				<PlotPage />
			</Route>
			<Route path="/timeline">
				<TimelinePage />
			</Route>
			<Route
				path="/category-chart-deep/:prefix"
				render={(
					r: RouteComponentProps<{
						prefix: string
					}>,
				) => (
					<ChooserWithChild
						child={(p) => (
							<CategoryChart
								{...p}
								deep
								tagPrefix={r.match.params.prefix}
							/>
						)}
					/>
				)}
			/>
			<Route
				path="/category-chart/:prefix"
				render={(
					r: RouteComponentProps<{
						prefix: string
					}>,
				) => (
					<ChooserWithChild
						child={(p) => (
							<CategoryChart
								{...p}
								deep={false}
								tagPrefix={r.match.params.prefix}
							/>
						)}
					/>
				)}
			/>
			<Route path="/tag-tree">
				<TagTreePage />
			</Route>
			<Route exact path="/">
				<Redirect to="/timeline"></Redirect>
			</Route>
			<Route
				path="/single-event/:id"
				render={(p: RouteComponentProps<{ id: string }>) => (
					<SingleEventInfo id={p.match.params.id}></SingleEventInfo>
				)}
			></Route>
			<div>Error 404</div>
		</Switch>
	)
}
