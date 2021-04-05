/*!

=========================================================
* Argon Dashboard React - v1.2.0
=========================================================

* Product Page: https://www.creative-tim.com/product/argon-dashboard-react
* Copyright 2021 Creative Tim (https://www.creative-tim.com)
* Licensed under MIT (https://github.com/creativetimofficial/argon-dashboard-react/blob/master/LICENSE.md)

* Coded by Creative Tim

=========================================================

* The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

*/
import React from "react"
// reactstrap components
import {
	DropdownMenu,
	DropdownItem,
	UncontrolledDropdown,
	DropdownToggle,
	Form,
	FormGroup,
	InputGroupAddon,
	InputGroupText,
	Input,
	InputGroup,
	Navbar,
	Nav,
	Container,
	Media,
	NavbarBrand,
	NavItem,
	NavLink as NL2,
} from "reactstrap"
import { NavLink } from "../router-lib/components/NavLink"
import { routes } from "../routes"

export const MyNavbar = (props) => {
	return (
		<>
			<Navbar
				className="navbar-top navbar-dark"
				expand="md"
				id="navbar-main"
			>
				<Container fluid>
					<NavbarBrand href="/">Timetrackrs</NavbarBrand>

					<Nav className="mr-auto" navbar>
						<NavItem>
							<NavLink
								route={routes.dashboard}
								args={{}}
								query={{}}
							>
								Dashboard
							</NavLink>
						</NavItem>
						<NavItem>
							<NavLink
								route={routes.timeline}
								args={{}}
								query={{}}
							>
								Timeline
							</NavLink>
						</NavItem>
						<NavItem>
							<NavLink
								route={routes.tagTree}
								args={{}}
								query={{}}
							>
								Tag Tree
							</NavLink>
						</NavItem>
						<NavItem>
							<NavLink route={routes.plot} args={{}} query={{}}>
								Plot
							</NavLink>
						</NavItem>
						<NavItem>
							<NavLink
								route={routes.ruleEditor}
								args={{}}
								query={{}}
							>
								Rule Editor
							</NavLink>
						</NavItem>
						{/*<UncontrolledDropdown nav>
							<DropdownToggle className="pr-0" nav>
								<Media className="align-items-center">
									<span className="avatar avatar-sm rounded-circle">
										<img
											alt="..."
											src={
												require("../../assets/img/theme/team-4-800x800.jpg")
													.default
											}
										/>
									</span>
									<Media className="ml-2 d-none d-lg-block">
										<span className="mb-0 text-sm font-weight-bold">
											Jessica Jones
										</span>
									</Media>
								</Media>
                                        </DropdownToggle>
							<DropdownMenu className="dropdown-menu-arrow" right>
								<DropdownItem
									className="noti-title"
									header
									tag="div"
								>
									<h6 className="text-overflow m-0">
										Welcome!
									</h6>
								</DropdownItem>
								<DropdownItem
									to="/admin/user-profile"
									tag={Link}
								>
									<i className="ni ni-single-02" />
									<span>My profile</span>
								</DropdownItem>
								<DropdownItem
									to="/admin/user-profile"
									tag={Link}
								>
									<i className="ni ni-settings-gear-65" />
									<span>Settings</span>
								</DropdownItem>
								<DropdownItem
									to="/admin/user-profile"
									tag={Link}
								>
									<i className="ni ni-calendar-grid-58" />
									<span>Activity</span>
								</DropdownItem>
								<DropdownItem
									to="/admin/user-profile"
									tag={Link}
								>
									<i className="ni ni-support-16" />
									<span>Support</span>
								</DropdownItem>
								<DropdownItem divider />
								<DropdownItem
									href="#pablo"
									onClick={(e) => e.preventDefault()}
								>
									<i className="ni ni-user-run" />
									<span>Logout</span>
								</DropdownItem>
							</DropdownMenu>
                                        </UncontrolledDropdown>*/}
					</Nav>
				</Container>
			</Navbar>
		</>
	)
}
