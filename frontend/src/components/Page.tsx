import * as React from "react"
import { routes } from "../routes"
import { NavLink } from "../router-lib/components/NavLink"
import { Container } from "reactstrap"
import { Footer } from "./Footer"
import { MyNavbar } from "./Navbar"

export const Page: React.FC<{
	navRight?: React.ComponentType
}> = ({ children, navRight }) => {
	return (
		<div className="main-content">
			<MyNavbar navRight={navRight} />
			{children}
			<Container fluid>
				<Footer />
			</Container>
		</div>
	)
}
