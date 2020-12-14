import React from "react"
import { CategoryChart } from "./components/CategoryChart"
import { ChooserWithChild } from "./components/ChooserWithChild"
import { PlotPage } from "./components/Plot"
import { TagRuleEditorPage } from "./components/TagRuleEditor"
// import { Switch, Route, Redirect, RouteComponentProps } from "react-router-dom"
import { TagTreePage } from "./components/TagTree"
import { TimelinePage } from "./components/Timeline"
import { asQueryArgs, Route, Router, Routing } from "./router-lib"

const rootQueryArgs = asQueryArgs({
	server: "string",
})

export const routes = {
	plot: Route.create("/plot").withQueryArgs(rootQueryArgs),
	timeline: Route.create("/timeline").withQueryArgs(rootQueryArgs),
	tagTree: Route.create("/tag-tree").withQueryArgs(rootQueryArgs),
	ruleEditor: Route.create("/rule-editor").withQueryArgs(rootQueryArgs),
	categoryChart: Route.create("/category-chart/:tagName", {
		tagName: "string",
	})
		.withQueryArgs(rootQueryArgs)
		.withQueryArgs({ deep: "boolean" }),
}
export const timeline = Route.create("/timeline").withQueryArgs(rootQueryArgs)

export const router = Router.create<React.ComponentType>()
	.with(routes.plot, (p) => PlotPage)
	.with(routes.timeline, (p) => TimelinePage)
	.with(routes.tagTree, (p) => TagTreePage)
	.with(routes.ruleEditor, (p) => TagRuleEditorPage)
	.with(routes.categoryChart, (p1) => () => {
		console.log("foo", p1)
		return (
			<ChooserWithChild
				child={(p2) => (
					<CategoryChart
						{...p2}
						deep={!!p1.queryArgs.deep}
						tagName={p1.args.tagName}
					/>
				)}
			/>
		)
	})

export type RoutingType = Routing<
	typeof router["_tdata"],
	typeof router["_targs"]
>
export const RouterContext = React.createContext<null | RoutingType>(null)

/*return (
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
	)*/
