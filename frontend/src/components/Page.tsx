import * as React from "react"
import { NavLink } from "react-router-dom"

export const Page: React.FC<{
	headerChildren?: React.ReactNode
	title?: string
}> = ({ title, headerChildren, children }) => {
	return (
		<div className="container">
			<div className="header fade-in">
				<h1>TRBTT</h1>
				<span className="nav-bar">
					<NavLink to="/timeline">Timeline</NavLink>
					{" • "}
					<NavLink to="/tag-tree">Tag Tree</NavLink>
				</span>
				{headerChildren}
			</div>
			{children}
		</div>
	)
}
