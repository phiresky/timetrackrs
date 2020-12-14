import { createHashHistory } from "history"
import { observer, useLocalObservable } from "mobx-react"
import React, { createContext, useContext } from "react"
import Modal from "react-modal"
import {
	LocationService,
	QueryArgs,
	QueryArgsToType,
	Route,
	RouteArgs,
	RouteArgsToType,
	Routing,
} from "../router-lib"
import { Link } from "../router-lib/components/Link"
import { router, RouterContext, RoutingType } from "../routes"
import { Routes } from "./Routes"

const ModalContext = createContext(
	null as null | { routing: RoutingType; isOpen: boolean },
)
export function ModalLink<A extends RouteArgs, Q extends QueryArgs>(p: {
	route: Route<A, Q>
	args: RouteArgsToType<A>
	query: QueryArgsToType<Q>
	children?: React.ReactNode
	aProps?: { className: string }
}): React.ReactElement {
	const context = useContext(ModalContext)
	if (!context) return <a>[no modal context found]</a>
	return (
		<Link
			route={p.route}
			args={p.args}
			query={p.query}
			routing={context.routing}
		>
			{p.children}
		</Link>
	)
}

export const MaybeModal: React.FC<{ appElement: HTMLElement }> = observer(
	({ appElement, children }) => {
		const store = useLocalObservable(() => {
			const history = createHashHistory()
			const routing = new Routing(router, new LocationService(history))
			// modal is open if hash router path is not /
			return {
				history,
				routing,
				get isOpen() {
					return (
						routing.locationService.currentLocation.path.length > 0
					)
				},
				close() {
					this.history.push("/")
				},
			}
		})

		return (
			<ModalContext.Provider value={store}>
				{children}
				{store.isOpen && (
					<Modal
						isOpen={true}
						appElement={appElement}
						onRequestClose={(_) => store.close()}
					>
						<RouterContext.Provider value={store.routing}>
							<Routes />
						</RouterContext.Provider>
					</Modal>
				)}
			</ModalContext.Provider>
		)
	},
)
