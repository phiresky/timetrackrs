import { observable, runInAction } from "mobx"
import { Disposable } from "@hediet/std/disposable"
import { LocationInfo } from "./LocationInfo"
import { History, createBrowserHistory } from "history"

export interface ClickInfo {
	href: string
	onClick: (e: React.MouseEvent<HTMLElement>) => void
}

export class LocationService {
	public readonly dispose = Disposable.fn()

	@observable private _currentLocation: LocationInfo
	public get currentLocation(): LocationInfo {
		return this._currentLocation
	}

	private readonly history: History

	constructor() {
		this.history = createBrowserHistory()
		this.dispose.track({
			dispose: this.history.listen((e) => {
				runInAction("Update current location", () => {
					this._currentLocation = LocationInfo.fromHistoryLocation(e)
				})
			}),
		})
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
		this.history.goBack()
		return true
	}
}
