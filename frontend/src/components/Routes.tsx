import React from "react"
import { Switch, Route, Redirect, RouteComponentProps } from "react-router-dom"
import { TagTreePage } from "./TagTree"
import { TimelinePage } from "./Timeline"
import { SingleEventInfo } from "./SingleEventInfo"
import { ChooserWithChild } from "./ChooserWithChild"
import { CategoryChart } from "./CategoryChart"
import { PlotPage } from "./Plot"
import { TagRuleEditorPage } from "./TagRuleEditor"

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
				path="/category-chart-deep/:tag"
				render={(
					r: RouteComponentProps<{
						tag: string
					}>,
				) => (
					<ChooserWithChild
						child={(p) => (
							<CategoryChart
								{...p}
								deep
								tagName={r.match.params.tag}
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
				) => {
					console.log(r)
					r.location.search
					return (
						<ChooserWithChild
							child={(p) => (
								<CategoryChart
									{...p}
									deep={false}
									tagName={r.match.params.prefix}
								/>
							)}
						/>
					)
				}}
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
			<Route
				path="/tag-rule-editor"
				render={(p: RouteComponentProps) => <TagRuleEditorPage />}
			></Route>

			<div>Error 404</div>
		</Switch>
	)
}
