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

import "@fortawesome/fontawesome-free/css/all.min.css"
// javascipt plugin for creating charts
import { Chart } from "chart.js"
// node.js library that concatenates classes (strings)
import classnames from "classnames"
import * as dfn from "date-fns"
import { observer, useLocalObservable } from "mobx-react"
import React from "react"
// react plugin used to create charts
import { Bar } from "react-chartjs-2"
// reactstrap components
import {
	Button,
	Card,
	CardBody,
	CardHeader,
	CardTitle,
	Col,
	Container,
	Nav,
	NavItem,
	NavLink,
	Progress,
	Row,
	Table,
} from "reactstrap"
import { LoadEvents } from "../components/ChooserWithChild"
import { Page } from "../components/Page"
import { InnerPlot } from "../components/Plot"
import { SingleNumberValue } from "../components/SingleNumberValue"
import {
	TimeRangeSelector,
	TimeRangeSelectorSimple,
	TimeRangeTarget,
} from "../components/TimeRangeSelector"
// core components
import { chartOptions, parseOptions } from "./charts"

function NotEnoughDataPlot(p: { dark?: boolean }) {
	return (
		<table style={{ height: 400, width: "100%" }}>
			<tr>
				<td className="align-middle">
					<h2 className="text-muted text-center">Not enough data</h2>
				</td>
			</tr>
		</table>
	)
}
export const Dashboard: React.FC = observer((_) => {
	const store = useLocalObservable(() => ({
		timeRange: {
			from: dfn.startOfDay(new Date()),
			to: dfn.endOfDay(new Date()),
			mode: "day",
		} as TimeRangeTarget,
		deep: false,
		setDeep(n: boolean) {
			this.deep = n
		},
	}))

	if (window.Chart) {
		parseOptions(Chart, chartOptions())
	}

	return (
		<Page
			navRight={() => (
				<div className="pr-0 pt-1" style={{ color: "white" }}>
					Showing data for{" "}
					<TimeRangeSelectorSimple target={store.timeRange} />
				</div>
			)}
		>
			<div className="header bg-gradient-info pb-8 pt-7 pt-md-7">
				<Container fluid>
					<div className="header-body">
						{/* Card stats */}
						<Row>
							<Col lg="6" xl="3">
								<Card className="card-stats mb-4 mb-xl-0">
									<CardBody>
										<Row>
											<div className="col">
												<CardTitle
													tag="h5"
													className="text-uppercase text-muted mb-0"
												>
													Total tracked time
												</CardTitle>
												<span className="h2 font-weight-bold mb-0">
													<SingleNumberValue
														time={store.timeRange}
														fetchFilter="timetrackrs-tracked"
														calculation="timetrackrs-tracked"
														unit="duration"
													/>
												</span>
											</div>
											<Col className="col-auto">
												<div className="icon icon-shape bg-danger text-white rounded-circle shadow">
													<i className="fas fa-chart-bar" />
												</div>
											</Col>
										</Row>
										<p className="mt-3 mb-0 text-muted text-sm">
											<span className="text-success mr-2">
												<i className="fa fa-arrow-up" />{" "}
												3.48%
											</span>{" "}
											<span className="text-nowrap">
												Since last month
											</span>
										</p>
									</CardBody>
								</Card>
							</Col>
							<Col lg="6" xl="3">
								<Card className="card-stats mb-4 mb-xl-0">
									<CardBody>
										<Row>
											<div className="col">
												<CardTitle
													tag="h5"
													className="text-uppercase text-muted mb-0"
												>
													Time spent on computer
												</CardTitle>
												<span className="h2 font-weight-bold mb-0">
													<SingleNumberValue
														time={store.timeRange}
														fetchFilter="use-device"
														calculation={{
															tag: "use-device",
															value: "computer",
														}}
														unit="duration"
													/>
												</span>
											</div>
											<Col className="col-auto">
												<div className="icon icon-shape bg-warning text-white rounded-circle shadow">
													<i className="fas fa-chart-pie" />
												</div>
											</Col>
										</Row>
										<p className="mt-3 mb-0 text-muted text-sm">
											<span className="text-danger mr-2">
												<i className="fas fa-arrow-down" />{" "}
												3.48%
											</span>{" "}
											<span className="text-nowrap">
												Since last week
											</span>
										</p>
									</CardBody>
								</Card>
							</Col>
							<Col lg="6" xl="3">
								<Card className="card-stats mb-4 mb-xl-0">
									<CardBody>
										<Row>
											<div className="col">
												<CardTitle
													tag="h5"
													className="text-uppercase text-muted mb-0"
												>
													Uncategorized time
												</CardTitle>
												<span className="h2 font-weight-bold mb-0">
													<SingleNumberValue
														time={store.timeRange}
														calculation={{
															minus: [
																1,
																{
																	div: [
																		{
																			tag:
																				"category",
																		},
																		{
																			tag:
																				"timetrackrs-tracked",
																		},
																	],
																},
															],
														}}
														unit="percentage"
													/>
												</span>
											</div>
											<Col className="col-auto">
												<div className="icon icon-shape bg-yellow text-white rounded-circle shadow">
													<i className="fas fa-users" />
												</div>
											</Col>
										</Row>
										<p className="mt-3 mb-0 text-muted text-sm">
											<span className="text-warning mr-2">
												<i className="fas fa-arrow-down" />{" "}
												1.10%
											</span>{" "}
											<span className="text-nowrap">
												Since yesterday
											</span>
										</p>
									</CardBody>
								</Card>
							</Col>
							<Col lg="6" xl="3">
								<Card className="card-stats mb-4 mb-xl-0">
									<CardBody>
										<Row>
											<div className="col">
												<CardTitle
													tag="h5"
													className="text-uppercase text-muted mb-0"
												>
													Productivity
												</CardTitle>
												<span className="h2 font-weight-bold mb-0">
													<SingleNumberValue
														time={store.timeRange}
														fetchFilter="category"
														calculation={{
															div: [
																{
																	tag:
																		"category",
																	valuePrefix:
																		"Productivity/",
																},
																{
																	tag:
																		"category",
																},
															],
														}}
														unit="percentage"
													/>
												</span>
											</div>
											<Col className="col-auto">
												<div className="icon icon-shape bg-info text-white rounded-circle shadow">
													<i className="fas fa-percent" />
												</div>
											</Col>
										</Row>
										<p className="mt-3 mb-0 text-muted text-sm">
											<span className="text-success mr-2">
												<i className="fas fa-arrow-up" />{" "}
												12%
											</span>{" "}
											<span className="text-nowrap">
												Since last month
											</span>
										</p>
									</CardBody>
								</Card>
							</Col>
						</Row>
					</div>
				</Container>
			</div>
			{/* Page content */}
			<Container className="mt--7" fluid>
				<Row>
					<Col className="mb-5 mb-xl-0" xl="8">
						<Card className="bg-gradient-default shadow">
							<CardHeader className="bg-transparent">
								<Row className="align-items-center">
									<div className="col">
										<h6 className="text-uppercase text-light ls-1 mb-1">
											Time spent by category
										</h6>
										<h2 className="text-white mb-0">
											History
										</h2>
									</div>
									<div className="col">
										<Nav
											className="justify-content-end"
											pills
										>
											<NavItem>
												<NavLink
													className={classnames(
														"py-2 px-3",
														{
															active: !store.deep,
														},
													)}
													href="#"
													onClick={(e) => {
														e.preventDefault()
														store.setDeep(false)
													}}
												>
													<span className="d-none d-md-block">
														Simple
													</span>
													<span className="d-md-none">
														S
													</span>
												</NavLink>
											</NavItem>
											<NavItem>
												<NavLink
													className={classnames(
														"py-2 px-3",
														{
															active: store.deep,
														},
													)}
													data-toggle="tab"
													href="#"
													onClick={(e) => {
														e.preventDefault()
														store.setDeep(true)
													}}
												>
													<span className="d-none d-md-block">
														Detailed
													</span>
													<span className="d-md-none">
														D
													</span>
												</NavLink>
											</NavItem>
										</Nav>
									</div>
								</Row>
							</CardHeader>
							<CardBody>
								<LoadEvents
									timeRange={store.timeRange}
									tag="category"
									child={(p) => {
										if (p.events.length < 3)
											return (
												<NotEnoughDataPlot
													dark
												></NotEnoughDataPlot>
											)
										return (
											<InnerPlot
												events={p.events}
												tag={p.tag}
												binSize={20 * 1000 * 60}
												aggregator={{
													name: "none",
													mapper: (d) => d,
													visible: true,
												}}
												deep={store.deep}
												dark={true}
											/>
										)
									}}
								/>
								{/* Chart 
								<div className="chart">
									<Line
										data={chartExample1[chartExample1Data]}
										options={chartExample1.options}
										getDatasetAtEvent={(e) =>
											console.log(e)
										}
									/>
									</div>*/}
							</CardBody>
						</Card>
					</Col>
					<Col xl="4">
						<Card className="shadow">
							<CardHeader className="bg-transparent">
								<Row className="align-items-center">
									<div className="col">
										<h6 className="text-uppercase text-muted ls-1 mb-1">
											Time spent by category
										</h6>
										<h2 className="mb-0">Overview</h2>
									</div>
								</Row>
							</CardHeader>
							<CardBody>
								<LoadEvents
									timeRange={store.timeRange}
									tag="category"
									child={(p) => {
										if (p.events.length < 3)
											return <NotEnoughDataPlot />
										return (
											<InnerPlot
												events={p.events}
												chartType="pie"
												tag={p.tag}
												binSize={Infinity}
												aggregator={{
													name: "none",
													mapper: (d) => d,
													visible: true,
												}}
												deep={store.deep}
												dark={false}
											/>
										)
									}}
								/>
							</CardBody>
						</Card>
					</Col>
				</Row>
				<Row className="mt-5">
					<Col className="mb-5 mb-xl-0" xl="8">
						<Card className="shadow">
							<CardHeader className="border-0">
								<Row className="align-items-center">
									<div className="col">
										<h3 className="mb-0">Page visits</h3>
									</div>
									<div className="col text-right">
										<Button
											color="primary"
											href="#pablo"
											onClick={(e) => e.preventDefault()}
											size="sm"
										>
											See all
										</Button>
									</div>
								</Row>
							</CardHeader>
							<Table
								className="align-items-center table-flush"
								responsive
							>
								<thead className="thead-light">
									<tr>
										<th scope="col">Page name</th>
										<th scope="col">Visitors</th>
										<th scope="col">Unique users</th>
										<th scope="col">Bounce rate</th>
									</tr>
								</thead>
								<tbody>
									<tr>
										<th scope="row">/argon/</th>
										<td>4,569</td>
										<td>340</td>
										<td>
											<i className="fas fa-arrow-up text-success mr-3" />{" "}
											46,53%
										</td>
									</tr>
									<tr>
										<th scope="row">/argon/index.html</th>
										<td>3,985</td>
										<td>319</td>
										<td>
											<i className="fas fa-arrow-down text-warning mr-3" />{" "}
											46,53%
										</td>
									</tr>
									<tr>
										<th scope="row">/argon/charts.html</th>
										<td>3,513</td>
										<td>294</td>
										<td>
											<i className="fas fa-arrow-down text-warning mr-3" />{" "}
											36,49%
										</td>
									</tr>
									<tr>
										<th scope="row">/argon/tables.html</th>
										<td>2,050</td>
										<td>147</td>
										<td>
											<i className="fas fa-arrow-up text-success mr-3" />{" "}
											50,87%
										</td>
									</tr>
									<tr>
										<th scope="row">/argon/profile.html</th>
										<td>1,795</td>
										<td>190</td>
										<td>
											<i className="fas fa-arrow-down text-danger mr-3" />{" "}
											46,53%
										</td>
									</tr>
								</tbody>
							</Table>
						</Card>
					</Col>
					<Col xl="4">
						<Card className="shadow">
							<CardHeader className="border-0">
								<Row className="align-items-center">
									<div className="col">
										<h3 className="mb-0">Social traffic</h3>
									</div>
									<div className="col text-right">
										<Button
											color="primary"
											href="#pablo"
											onClick={(e) => e.preventDefault()}
											size="sm"
										>
											See all
										</Button>
									</div>
								</Row>
							</CardHeader>
							<Table
								className="align-items-center table-flush"
								responsive
							>
								<thead className="thead-light">
									<tr>
										<th scope="col">Referral</th>
										<th scope="col">Visitors</th>
										<th scope="col" />
									</tr>
								</thead>
								<tbody>
									<tr>
										<th scope="row">Facebook</th>
										<td>1,480</td>
										<td>
											<div className="d-flex align-items-center">
												<span className="mr-2">
													60%
												</span>
												<div>
													<Progress
														max="100"
														value="60"
														barClassName="bg-gradient-danger"
													/>
												</div>
											</div>
										</td>
									</tr>
									<tr>
										<th scope="row">Facebook</th>
										<td>5,480</td>
										<td>
											<div className="d-flex align-items-center">
												<span className="mr-2">
													70%
												</span>
												<div>
													<Progress
														max="100"
														value="70"
														barClassName="bg-gradient-success"
													/>
												</div>
											</div>
										</td>
									</tr>
									<tr>
										<th scope="row">Google</th>
										<td>4,807</td>
										<td>
											<div className="d-flex align-items-center">
												<span className="mr-2">
													80%
												</span>
												<div>
													<Progress
														max="100"
														value="80"
													/>
												</div>
											</div>
										</td>
									</tr>
									<tr>
										<th scope="row">Instagram</th>
										<td>3,678</td>
										<td>
											<div className="d-flex align-items-center">
												<span className="mr-2">
													75%
												</span>
												<div>
													<Progress
														max="100"
														value="75"
														barClassName="bg-gradient-info"
													/>
												</div>
											</div>
										</td>
									</tr>
									<tr>
										<th scope="row">twitter</th>
										<td>2,645</td>
										<td>
											<div className="d-flex align-items-center">
												<span className="mr-2">
													30%
												</span>
												<div>
													<Progress
														max="100"
														value="30"
														barClassName="bg-gradient-warning"
													/>
												</div>
											</div>
										</td>
									</tr>
								</tbody>
							</Table>
						</Card>
					</Col>
				</Row>
			</Container>
		</Page>
	)
})
