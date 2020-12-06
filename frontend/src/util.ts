import { Activity } from "./api"

export function totalDuration(entries: Activity[]): number {
	return entries.reduce((sum, b) => sum + b.duration, 0)
}

export function durationToString(duration: number): string {
	if (duration < 60) {
		return `${Math.round(duration)} s`
	}
	duration = Math.round(duration / 60)
	if (duration >= 60)
		return `${Math.round(duration / 60)} h ${duration % 60} min`
	return `${duration} min`
}

export class DefaultMap<K, V> extends Map<K, V> {
	constructor(private def: () => V, entries?: [K, V][]) {
		super(entries)
	}
	get(k: K): V {
		let res = super.get(k)
		if (!res) {
			res = this.def()
			this.set(k, res)
		}
		return res
	}
}
export class KeyedSet<E> implements Set<E> {
	private map = new Map<string, E>()
	constructor(private getKey: (e: E) => string) {}
	[Symbol.toStringTag]: string
	add(value: E): this {
		this.map.set(this.getKey(value), value)
		return this
	}
	clear(): void {
		this.map.clear()
	}
	delete(value: E): boolean {
		return this.map.delete(this.getKey(value))
	}
	forEach(
		callbackfn: (value: E, value2: E, set: Set<E>) => void,
		thisArg?: any,
	): void {
		this.map.forEach((v, k) => callbackfn(v, v, this))
	}
	has(value: E): boolean {
		return this.map.has(this.getKey(value))
	}
	get size(): number {
		return this.map.size
	}
	[Symbol.iterator](): IterableIterator<E> {
		return this.map.values()
	}
	entries(): IterableIterator<[E, E]> {
		throw new Error("Method not implemented.")
	}
	keys(): IterableIterator<E> {
		return this.map.values()
	}
	values(): IterableIterator<E> {
		return this.map.values()
	}
}
