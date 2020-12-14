import { LocationInfo } from "./LocationInfo"
import { compile, match } from "path-to-regexp"

export interface Matcher<T> {
	matches(location: LocationInfo): T | undefined
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
	TQueryArgs extends QueryArgs
> {
	args: RouteArgsToType<TArgs>
	queryArgs: QueryArgsToType<TQueryArgs>
}

export class Route<TArgs extends RouteArgs, TQueryArgs extends QueryArgs>
	implements Matcher<MatcherArgs<TArgs, TQueryArgs>> {
	public static create(path: string): Route<{}, {}>
	public static create<TArgs extends RouteArgs>(
		path: string,
		args: TArgs,
	): Route<TArgs, {}>
	public static create<TArgs extends RouteArgs = {}>(
		path: string,
		args?: TArgs,
	): Route<TArgs, {}> {
		return new Route(path, args!, {})
	}

	private readonly matchFn = match(this.path)
	private readonly compileFn = compile(this.path)

	private constructor(
		public readonly path: string,
		public readonly args: TArgs,
		public readonly queryArgs: TQueryArgs,
	) {}

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
		return LocationInfo.parse(path)
	}

	public matches(
		location: LocationInfo,
	):
		| {
				args: RouteArgsToType<TArgs>
				queryArgs: QueryArgsToType<TQueryArgs>
		  }
		| undefined {
		const r = this.matchFn(location.getPathString())
		if (r) {
			const params = new URLSearchParams(location.search)
			const args: Record<string, any> = {}
			for (const [key, value] of params) {
				args[key] = value // TODO
			}
			return {
				args: r.params as RouteArgsToType<TArgs>,
				queryArgs: args as any,
			}
		} else {
			return undefined
		}
	}
}
