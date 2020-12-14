import { observer, useLocalObservable } from "mobx-react"
import React, { useContext } from "react"
import { LocationService, Routing } from "../router-lib"
import { router, RouterContext } from "../routes"

export const Routes: React.FC = observer(() => {
	const routing = useContext(RouterContext)
	if (!routing) return <div>[no router provider]</div>
	const currentRoute = routing.currentRouteInformation
	if (currentRoute) {
		const E = currentRoute.data
		return <E />
	} else {
		return <div>404!</div>
	}
})

export const BrowserRouterProvider: React.FC = observer(({ children }) => {
	const model = useLocalObservable(() => ({
		routing: new Routing(router, new LocationService()),
	}))
	return (
		<RouterContext.Provider value={model.routing}>
			{children}
		</RouterContext.Provider>
	)
})
