import * as React from "react"
import { useContext } from "react"
import {
	QueryArgs,
	QueryArgsToType,
	Route,
	RouteArgs,
	RouteArgsToType,
} from ".."
import { RouterContext, RoutingType } from "../../routes"

export function Link<A extends RouteArgs, Q extends QueryArgs>(p: {
	route: Route<A, Q>
	args: RouteArgsToType<A>
	query: QueryArgsToType<Q>
	children?: React.ReactNode
	aProps?: React.AnchorHTMLAttributes<HTMLAnchorElement>
	routing?: RoutingType
}): React.ReactElement {
	const routing = p.routing || useContext(RouterContext)
	if (!routing) return <a>[router gone]</a>
	return (
		<a
			{...(p.aProps || {})}
			{...routing.locationToOnClick(p.route.build(p.args, p.query))}
		>
			{p.children}
		</a>
	)
}
