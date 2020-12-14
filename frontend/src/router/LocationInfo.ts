import * as history from "history"

function searchToObj(search: string): Record<string, string> {
	const p = new URLSearchParams(search)
	return Object.fromEntries(p)
}

function objToSearch(obj: Record<string, string>): string {
	return new URLSearchParams(obj).toString()
}

export interface SearchData {
	readonly [key: string]: string
}

export class LocationInfo {
	public static parse(path: string): LocationInfo {
		const p = history.parsePath(path)
		return LocationInfo.fromHistoryLocation(p)
	}

	private static parsePath(path: string): string[] {
		if (!path.startsWith("/")) {
			throw new Error(`Expected pathname "${path}" to start with "/"`)
		}
		path = path.substr(1)
		const items = path ? path.split("/") : []
		return items
	}

	public static fromHistoryLocation(
		location: history.Location,
	): LocationInfo {
		const path = LocationInfo.parsePath(location.pathname)
		return new LocationInfo(
			path,
			searchToObj(location.search),
			location.hash,
			location.state,
		)
	}

	public static deserialize(data: string): LocationInfo {
		const obj = JSON.parse(data)
		return new LocationInfo(obj.path, obj.search, obj.hash, obj.state)
	}

	constructor(
		public readonly path: string[],
		public readonly search: SearchData,
		public readonly hash: string,
		/** Has to serialize/deserialize to/from json */
		public readonly state: unknown,
	) {}

	public getPathString(): string {
		return "/" + this.path.join("/")
	}

	public toHistoryLocation(): history.Location {
		return {
			pathname: this.getPathString(),
			hash: this.hash,
			state: this.state as any,
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
		let str = this.getPathString()
		const search = objToSearch(this.search)
		if (search !== "") {
			str += `?${search}`
		}
		return str
	}
}

export interface LocationMatch {
	params: Record<string, string>
}
