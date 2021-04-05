import * as React from "react"
import { useContext } from "react"
import {
	QueryArgs,
	QueryArgsToType,
	Route,
	RouteArgs,
	RouteArgsToType,
} from ".."
import { RouterContext } from "../../routes"
import { Link } from "./Link"

export function NavLink<A extends RouteArgs, Q extends QueryArgs>(p: {
	route: Route<A, Q>
	args: RouteArgsToType<A>
	query: QueryArgsToType<Q>
	children?: React.ReactNode
}): React.ReactElement {
	const routing = useContext(RouterContext)
	if (!routing) return <a>[router gone]</a>
	const isCurrent = routing.currentRouteInformation?.matcher === p.route
	return (
		<Link
			{...p}
			aProps={{ className: isCurrent ? "active nav-link" : "nav-link" }}
		/>
	)
}
