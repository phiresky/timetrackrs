import * as history from "history"

function searchToObj(search: string | undefined): Record<string, string> {
	if (!search) return {}
	const p = new URLSearchParams(search)
	return Object.fromEntries(p)
}

function objToSearch(obj: Record<string, string>): string {
	const str = new URLSearchParams(obj).toString()
	if (str) return "?" + str
	return str
}

export interface SearchData {
	readonly [key: string]: string
}

export class LocationInfo {
	public static parsePath(path: string | undefined): string[] {
		if (!path) return []
		if (!path.startsWith("/")) {
			throw new Error(`Expected pathname "${path}" to start with "/"`)
		}
		path = path.substr(1)
		const items = path ? path.split("/") : []
		return items
	}

	public static fromHistoryLocation(
		location: history.PartialPath | history.Location,
	): LocationInfo {
		const path = LocationInfo.parsePath(location.pathname)
		return new LocationInfo(
			path,
			searchToObj(location.search),
			location.hash || "",
			"state" in location
				? (location.state as Record<string, unknown>)
				: null,
		)
	}

	public static deserialize(data: string): LocationInfo {
		const obj = JSON.parse(data) as {
			path: string[]
			search: SearchData
			hash: string
			state: Record<string, unknown> | null
		}
		return new LocationInfo(obj.path, obj.search, obj.hash, obj.state)
	}

	constructor(
		public readonly path: string[],
		public readonly search: SearchData,
		public readonly hash: string,
		/** Has to serialize/deserialize to/from json */
		public readonly state: Record<string, unknown> | null,
	) {}

	public getPathString(): string {
		return "/" + this.path.join("/")
	}

	public toHistoryLocation(): history.Location {
		return {
			key: "default",
			pathname: this.getPathString(),
			hash: this.hash,
			state: this.state,
			search: objToSearch(this.search),
		}
	}

	public equals(other: LocationInfo): boolean {
		return this.serialize() === other.serialize()
	}

	public withPath(path: string): LocationInfo {
		return new LocationInfo(
			LocationInfo.parsePath(path),
			this.search,
			this.hash,
			this.state,
		)
	}

	public withSearch(search: SearchData): LocationInfo {
		return new LocationInfo(this.path, search, this.hash, this.state)
	}

	public serialize(): string {
		return JSON.stringify({
			path: this.path,
			search: this.search,
			hash: this.hash,
			state: this.state,
		})
	}

	public toString(): string {
		return this.getPathString() + objToSearch(this.search)
	}
}

export interface LocationMatch {
	params: Record<string, string>
}
