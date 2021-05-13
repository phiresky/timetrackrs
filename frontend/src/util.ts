import { autorun } from "mobx"
import React from "react"
import { SingleExtractedChunk } from "./server"

export function totalDurationSeconds(entries: SingleExtractedChunk[]): number {
	return entries.reduce((sum, b) => sum + b.to_exclusive - b.from, 0) / 1000
}

export function durationToString(duration: number): string {
	if (duration < 60) {
		return `${Math.round(duration)} s`
	}
	duration = Math.round(duration / 60)
	if (duration >= 60)
		return `${Math.floor(duration / 60)} h ${duration % 60} min`
	return `${duration} min`
}

/** same as useEffect, but dependencies determined by mobx instead of manually */
export function useMobxEffect(effect: () => unknown): void {
	return React.useEffect(() => autorun(effect), [])
}

export function expectNever<T = any>(n: never): T {
	return n as T
}

export class NeatMap<K, V> extends Map<K, V> {
	addDelta(this: NeatMap<K, number>, k: K, delta: number): number {
		const newValue = (this.get(k) || 0) + delta
		this.set(k, newValue)
		return newValue
	}
}
export class DefaultMap<K, V> extends NeatMap<K, V> {
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

export class Counter<K> extends DefaultMap<K, number> {
	constructor(entries?: K[]) {
		super(() => 0)
		if (entries)
			for (const entry of entries) {
				this.add(entry)
			}
	}
	add(k: K): void {
		this.set(k, this.get(k) + 1)
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

// generateId :: Integer -> String
export function generateId(len = 16): string {
	const arr = new Uint8Array(len / 2)
	window.crypto.getRandomValues(arr)
	return Array.from(arr, (dec) => dec.toString(16).padStart(2, "0")).join("")
}

export function intersperse<T>(arr: T[], separator: (n: number) => T): T[] {
	return arr.flatMap((a, i) => (i > 0 ? [separator(i - 1), a] : [a]))
}
