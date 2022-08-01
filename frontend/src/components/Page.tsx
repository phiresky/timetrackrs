import * as React from "react"
import { Container } from "reactstrap"
import { Footer } from "./Footer"
import { MyNavbar } from "./Navbar"

export const Page: React.FC<
	React.PropsWithChildren<{
		navRight?: React.ComponentType
		title?: string
	}>
> = ({ children, navRight }) => {
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
