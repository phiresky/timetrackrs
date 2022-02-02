import { action, makeObservable, observable } from "mobx"
import { LocationInfo } from "./LocationInfo"
import { History, createBrowserHistory } from "history"

export interface ClickInfo {
	href: string
	onClick: (e: React.MouseEvent<HTMLElement>) => void
}

export class LocationService {
	@observable private _currentLocation!: LocationInfo
	public get currentLocation(): LocationInfo {
		return this._currentLocation
	}

	private readonly history: History

	constructor(history: History = createBrowserHistory()) {
		makeObservable(this)

		this.history = history
		// todo: dispose
		this.history.listen((e) => {
			console.log("hist change, now at", e.action, e.location)
			this.updateLocationFromHistory(e.location)
		})

		this.updateLocationFromHistory()
	}
	@action
	private updateLocationFromHistory(e = this.history.location) {
		this._currentLocation = LocationInfo.fromHistoryLocation(
			this.history.location,
		)
	}

	public locationToOnClick(location: LocationInfo): ClickInfo {
		return {
			onClick: (e: React.MouseEvent<HTMLElement>) => {
				e.preventDefault()
				this.push(location)
			},
			href: location.toString(),
		}
	}

	public pushPath(path: string): void {
		const loc = this.currentLocation.withPath(path)
		this.push(loc)
	}

	public replacePath(path: string): void {
		const loc = this.currentLocation.withPath(path)
		this.replace(loc)
	}

	public push(newLocation: LocationInfo): void {
		this.history.push(newLocation.toHistoryLocation())
	}

	public replace(newLocation: LocationInfo): void {
		this.history.replace(newLocation.toHistoryLocation())
	}

	public pop(): boolean {
		this.history.back()
		return true
	}
}
