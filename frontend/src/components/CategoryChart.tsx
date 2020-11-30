import { computed } from "mobx"
import React from "react"
import Plot from "react-plotly.js"
import { inflateRawSync } from "zlib"
import { Activity } from "../api"
import { totalDuration } from "../util"

class DefaultMap<K, V> extends Map<K, V> {
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
class KeyedSet<E> implements Set<E> {
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
export class CategoryChart extends React.Component<{
	events: Activity[]
	tagPrefix: string
	deep: boolean
}> {
	@computed get data() {
		const prefix = this.props.tagPrefix
		const groups = new DefaultMap<string, KeyedSet<Activity>>(
			() => new KeyedSet((e) => e.id),
		)
		for (const event of this.props.events) {
			for (const tag of event.data.tags) {
				if (tag.startsWith(prefix)) {
					let cat = tag.slice(prefix.length)
					if (!this.props.deep) cat = cat.split("/")[0]
					groups.get(cat).add(event)
				}
			}
		}
		const x = [...groups.keys()]
		const y = [...groups.values()].map(
			(s) => totalDuration([...s]) / 60 / 60,
		)
		return { x, y }
	}
	render(): React.ReactNode {
		return (
			<div>
				<Plot
					data={[
						{
							type: "bar",

							...this.data,
						},
					]}
					layout={{
						width: 700,
						yaxis: {
							title: "Hours",
						},
						height: 400,
						title: `Time spent per ${this.props.tagPrefix}`,
					}}
				/>
			</div>
		)
	}
}
