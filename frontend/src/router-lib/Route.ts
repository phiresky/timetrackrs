import { LocationInfo, SearchData } from "./LocationInfo"
import { compile, match, MatchFunction, PathFunction } from "path-to-regexp"
import { Routing } from "./Routing"

export interface Matcher<T> {
	matches(location: LocationInfo, routing: Routing<any, any>): T | undefined
}

export type RouteArgs = Record<string, "string" | "number">
export type QueryArgs = Record<string, "string" | "number" | "boolean">

export type RouteArgsToType<T extends RouteArgs> = {
	[TKey in keyof T]: {
		string: string
		number: number
	}[T[TKey]]
}

export type QueryArgsToType<T extends QueryArgs> = {
	[TKey in keyof T]?: {
		string: string
		number: number
		boolean: boolean
	}[T[TKey]]
}

export function asQueryArgs<T extends QueryArgs>(queryArgs: T): T {
	return queryArgs
}

export interface MatcherArgs<
	TArgs extends RouteArgs,
	TQueryArgs extends QueryArgs,
> {
	args: RouteArgsToType<TArgs>
	queryArgs: QueryArgsToType<TQueryArgs>
	replace: (
		route: Route<TArgs, TQueryArgs> | undefined,
		args: RouteArgsToType<TArgs> | undefined,
		queryArgs: QueryArgsToType<TQueryArgs> | undefined,
	) => void
}

export class Route<TArgs extends RouteArgs, TQueryArgs extends QueryArgs>
	implements Matcher<MatcherArgs<TArgs, TQueryArgs>>
{
	public static create(path: string): Route<RouteArgs, QueryArgs>
	public static create<TArgs extends RouteArgs>(
		path: string,
		args: TArgs,
	): Route<TArgs, QueryArgs>
	public static create<TArgs extends RouteArgs = RouteArgs>(
		path: string,
		args?: TArgs,
	): Route<TArgs, QueryArgs> {
		return new Route(path, args!, {})
	}

	private readonly matchFn: MatchFunction
	private readonly compileFn: PathFunction

	private constructor(
		public readonly path: string,
		public readonly args: TArgs,
		public readonly queryArgs: TQueryArgs,
	) {
		this.matchFn = match(this.path)
		this.compileFn = compile(this.path)
	}

	public withQueryArgs<TNewQueryArgs extends QueryArgs>(
		newArgs: TNewQueryArgs,
	): Route<TArgs, TQueryArgs & TNewQueryArgs> {
		return new Route<TArgs, TQueryArgs & TNewQueryArgs>(
			this.path,
			this.args,
			{
				...this.queryArgs,
				...newArgs,
			},
		)
	}

	public build(
		data: RouteArgsToType<TArgs>,
		queryArgs?: QueryArgsToType<TQueryArgs>,
	): LocationInfo {
		const path = this.compileFn(data)
		return new LocationInfo(
			LocationInfo.parsePath(path),
			queryArgs as SearchData,
			"",
			null,
		)
	}

	public matches(
		location: LocationInfo,
		routing: Routing<any, any>,
	): MatcherArgs<TArgs, TQueryArgs> | undefined {
		const r = this.matchFn(location.getPathString()) as
			| { params: RouteArgsToType<TArgs> }
			| false
		if (r) {
			const params = new URLSearchParams(location.search)
			const args: Record<string, any> = {}
			for (const [key, value] of params) {
				args[key] = value // TODO deserialize
			}
			const Oroute = this
			return {
				args: r.params,
				queryArgs: args as QueryArgsToType<TQueryArgs>,
				replace(route, args, queryArgs) {
					console.log("routing replace", route, args, queryArgs)
					routing.replace(
						route || Oroute,
						args || this.args,
						queryArgs || this.queryArgs,
					)
				},
			}
		} else {
			return undefined
		}
	}
}
