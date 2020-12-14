import { LocationInfo } from "./LocationInfo"
import { Matcher } from "./Route"

interface AddedRoute<TData> {
	matcher: Matcher<{}>
	dataProvider: (args: any) => TData
}

export interface RouteInformation<TData, TArgs> {
	args: TArgs
	data: TData
	matcher: Matcher<any>
}

export class Router<TData, TArgs> {
	public static create<TData>(): Router<TData, {}> {
		return new Router(undefined, undefined)
	}

	private constructor(
		private readonly parent: Router<any, any> | undefined,
		private readonly addedRoute: AddedRoute<TData> | undefined,
	) {}

	public with<TNewArgs>(
		matcher: Matcher<TNewArgs>,
		dataProvider: (args: TNewArgs) => TData,
	): Router<TData, TArgs & TNewArgs> {
		return new Router(this, { matcher, dataProvider })
	}

	public route(
		locationInfo: LocationInfo,
	): RouteInformation<TData, TArgs> | undefined {
		if (this.addedRoute) {
			const args = this.addedRoute.matcher.matches(locationInfo)
			if (args) {
				return {
					args: args as any,
					data: this.addedRoute.dataProvider(args),
					matcher: this.addedRoute.matcher,
				}
			}
		}
		if (this.parent) {
			return this.parent.route(locationInfo)
		}
		return undefined
	}
}

/*
const globalQueryArgs = asQueryArgs({
	serverUrl: "string",
});

const myRoute1 = Route.create("/foo/:id", { id: "string" }).withQueryArgs({
	...globalQueryArgs,
	bla: "string",
});

const router = new Router<{ result: string }>().with(myRoute1, data => ({
	result: data.args.id + data.queryArgs.bla,
}));

router.route(null!)?.args.queryArgs.serverUrl;
*/
//.create([myRoute1]);
