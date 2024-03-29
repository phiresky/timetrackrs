import React, { useContext } from "react"
import { CategoryChart } from "./components/CategoryChart"
import { ChooserWithChild } from "./components/ChooserWithChild"
import { Page } from "./components/Page"
import { PlotPage } from "./components/Plot"
import { SingleEventInfoFetch } from "./components/SingleEventInfo"
import { TagRuleEditorPage } from "./components/TagRuleEditor"
// import { Switch, Route, Redirect, RouteComponentProps } from "react-router-dom"
import { TagTreePage } from "./components/TagTree"
import { TimelinePage } from "./components/Timeline"
import { Dashboard } from "./pages/Dashboard"
import { asQueryArgs, Route, Router, Routing } from "./router-lib"

const rootQueryArgs = asQueryArgs({
	server: "string",
})

const chooserQueryArgs = asQueryArgs({
	from: "string",
	to: "string",
	tag: "string",
})

export const routes = {
	root: Route.create("/").withQueryArgs(rootQueryArgs),
	dashboard: Route.create("/dashboard"),
	plot: Route.create("/plot")
		.withQueryArgs(rootQueryArgs)
		.withQueryArgs(chooserQueryArgs),
	timeline: Route.create("/timeline").withQueryArgs(rootQueryArgs),
	tagTree: Route.create("/tag-tree")
		.withQueryArgs(rootQueryArgs)
		.withQueryArgs(chooserQueryArgs),
	ruleEditor: Route.create("/rule-editor").withQueryArgs(rootQueryArgs),
	categoryChart: Route.create("/category-chart")
		.withQueryArgs(rootQueryArgs)
		.withQueryArgs(chooserQueryArgs)
		.withQueryArgs({ deep: "boolean" }),
	singleEvent: Route.create("/single-event/:id", {
		id: "string",
	}).withQueryArgs(rootQueryArgs),
}
export const timeline = Route.create("/timeline").withQueryArgs(rootQueryArgs)

export const router = Router.create<React.ComponentType>()
	.with(routes.root, () => () => {
		// TODO: this redirect be ugly
		const c = useContext(RouterContext)
		c?.replace(routes.dashboard, {}, {})
		return <></>
	})
	.with(routes.plot, (p) => () => <PlotPage routeMatch={p} />)
	.with(routes.timeline, (p) => TimelinePage)
	.with(routes.tagTree, (p) => () => <TagTreePage routeMatch={p} />)
	.with(routes.ruleEditor, (p) => TagRuleEditorPage)
	.with(routes.categoryChart, (p) => () => {
		return (
			<ChooserWithChild
				routeMatch={p}
				chooseTag={true}
				child={(p2) => (
					<CategoryChart {...p2} deep={!!p.queryArgs.deep} />
				)}
			/>
		)
	})
	.with(routes.singleEvent, (p) => () => (
		<SingleEventInfoFetch id={p.args.id} />
	))
	.with(routes.dashboard, (p) => () => <Dashboard />)

export type RoutingType = Routing<
	typeof router["_tdata"],
	typeof router["_targs"]
>
export const RouterContext = React.createContext<null | RoutingType>(null)
