import * as React from "react"
import { routes } from "../routes"
import { NavLink } from "../router-lib/components/NavLink"

export const Page: React.FC<{
	headerChildren?: React.ReactNode
	title?: string
	headerClass?: string
	containerClass?: string
}> = ({ title, containerClass, headerChildren, children, headerClass }) => {
	return (
		<div className={`container ${containerClass || ""}`}>
			<div className={`header ${headerClass || ""}`}>
				<h1>TRBTT</h1>
				<span className="nav-bar">
					<NavLink route={routes.timeline} args={{}} query={{}}>
						Timeline
					</NavLink>
					{" • "}
					<NavLink route={routes.tagTree} args={{}} query={{}}>
						Tag Tree
					</NavLink>
					{" • "}
					<NavLink route={routes.plot} args={{}} query={{}}>
						Plot
					</NavLink>
					{" • "}
					<NavLink route={routes.ruleEditor} args={{}} query={{}}>
						Rule Editor
					</NavLink>
				</span>
				{headerChildren}
			</div>
			{children}
		</div>
	)
}
