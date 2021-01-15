import _ from "lodash"
import { computed, makeObservable } from "mobx"
import { observer, useLocalObservable } from "mobx-react"
import * as React from "react"
import { useState } from "react"
import { setTextRange } from "typescript"
import * as api from "../api"
import { SingleExtractedEvent } from "../server"
import {
	Counter,
	DefaultMap,
	durationToString,
	totalDurationSeconds,
} from "../util"
import {
	CategoryChart,
	CategoryChartModal as CategoryChartModalLink,
} from "./CategoryChart"
import { ChooserWithChild, CWCRouteMatch } from "./ChooserWithChild"
import { Entry } from "./Entry"
import { ModalLink } from "./ModalLink"
import { Page } from "./Page"
import { Choices, Select } from "./Select"

interface Tree<T> {
	leaves: T[]
	children: Map<string, Tree<T>>
}
type ATree = Tree<SingleExtractedEvent>

function rootTree<T>(): Tree<T> {
	return { children: new Map<string, Tree<T>>(), leaves: [] }
}

function addToTree<T>(t: Tree<T>, path: string[], leaf: T) {
	if (path.length === 0) {
		t.leaves.push(leaf)
		return
	}
	const [head, ...tail] = path
	let seg = t.children.get(head)
	if (!seg) {
		seg = rootTree()
		t.children.set(head, seg)
	}
	addToTree(seg, tail, leaf)
}

function shortenTree<T>(t: Tree<T>) {
	for (const [name, tree] of t.children) {
		if (tree.children.size === 1 && tree.leaves.length === 0) {
			const [innerName, innerChildren] = [...tree.children][0]
			t.children.delete(name)
			t.children.set(name + "/" + innerName, innerChildren)
		}
		shortenTree(tree)
	}
}

function sortTree(t: ATree, cache?: WeakMap<ATree, number>) {
	if (!cache) cache = new WeakMap()

	const sortKey = ([_, t]: [string, ATree]) => {
		let v = cache?.get(t)
		if (!v) {
			v = -totalDurationSeconds(collect(t))
			cache?.set(t, v)
		}
		return v
	}

	t.children = new Map(_.sortBy([...t.children], sortKey))
	for (const c of t.children) sortTree(c[1], cache)
}
/*

/a/b/foo
/a/b/bar

/: {a: {b: {foo, bar}}}

a -> b -> foo
       -> bar

/a/b -> foo
/a/b -> bar


*/

function collectRecurse(tree: ATree, add: (e: SingleExtractedEvent) => void) {
	tree.leaves.forEach(add)
	for (const c of tree.children.values()) collectRecurse(c, add)
}
function collect(tree: ATree) {
	const map = new Map<string, SingleExtractedEvent>()
	collectRecurse(tree, (e) => map.set(e.id, e))
	return [...map.values()]
}

function TotalDuration(props: { tree: ATree }) {
	return (
		<span>
			{durationToString(totalDurationSeconds(collect(props.tree)))}
		</span>
	)
}

const TreeLeaves: React.FC<{ leaves: SingleExtractedEvent[] }> = observer(
	({ leaves }) => {
		const [children, setChildren] = React.useState(5)
		const store = useLocalObservable(() => {
			const totalCount = leaves.length
			const totalCounts = new Counter<string>()
			const valueCounter = new DefaultMap<string, Counter<string>>(
				() => new Counter(),
			)
			for (const l of leaves) {
				for (const [tagKey, tagValues = []] of Object.entries(
					l.tags.map,
				)) {
					for (const value of tagValues) {
						totalCounts.add(tagKey)
						valueCounter.get(tagKey).add(value)
					}
				}
			}
			// somewhat incorrect: the total counts differ on each tags because events can have multiple tags
			// const correctTotalCount = _.sum([...totalCounts.values()])

			for (const [tagKey, counter] of valueCounter) {
				counter.set("[none]", totalCount - totalCounts.get(tagKey))
			}
			const averageEntropy = new Map(
				[...valueCounter].map(([tagKey, counter]) => {
					// sample probability
					const P = (count: number) => count / totalCount
					const entropy = -_.sumBy([...counter.values()], (count) =>
						count > 0 ? P(count) * Math.log2(P(count)) : 0,
					)
					console.log(tagKey, entropy)
					const entropyPerChoice = entropy / counter.size
					return [tagKey, entropyPerChoice] as const
				}),
			)
			const choicesList = _.sortBy([...averageEntropy], (k) => -k[1])
				.map((k) => ({
					value: k[0],
					name: `${k[0]} (${k[1].toFixed(2)})`,
				}))
				.slice(0, 40)
			return {
				choices: Choices(
					[
						{ value: "singles", name: "singles (no agg)" },
						...choicesList,
					],
					choicesList[0],
				),
			}
		})
		let inner
		if (store.choices.value.value === "singles")
			inner = (
				<ul>
					{leaves.slice(0, children).map((l) => (
						<li key={l.id}>
							<Entry {...l} />
						</li>
					))}
					{leaves.length > children && (
						<li key="more" className="clickable">
							<a
								className="clickable"
								onClick={() => setChildren(children * 2)}
							>
								...{leaves.length - children} more
							</a>
						</li>
					)}
				</ul>
			)
		else
			inner = (
				<TagTree events={leaves} tagName={store.choices.value.value} />
			)
		return (
			<div>
				{leaves.length} events. grouping by{" "}
				<Select<{ value: string; name: string }>
					target={store.choices}
					getValue={(e) => e.value}
					getName={(e) => e.name}
				/>
				{inner}
			</div>
		)
	},
)
function ShowTree({
	tag,
	tree,
	noSlash = false,
}: {
	tag: string
	tree: ATree
	noSlash?: boolean
}) {
	const [open, setOpen] = React.useState(false)

	const title = (noSlash ? "" : "/") + tag || "[empty]"

	return (
		<li key={tag}>
			<span className="clickable" onClick={() => setOpen(!open)}>
				{title} (<TotalDuration tree={tree} />)
			</span>

			{open && <ShowTreeChildren tree={tree} />}
			{open && tree.leaves.length > 0 && (
				<TreeLeaves leaves={tree.leaves} />
			)}
		</li>
	)
}
function ShowTreeChildren({
	tree,
	noSlash,
}: {
	tree: ATree
	noSlash?: boolean
}) {
	const [children, setChildren] = React.useState(5)
	return (
		<ul>
			{[...tree.children.entries()]
				.slice(0, children)
				.map(([tag, tree]) => (
					<ShowTree
						key={tag}
						tag={tag}
						tree={tree}
						noSlash={noSlash}
					/>
				))}
			{tree.children.size > children && (
				<li key="more" className="clickable">
					<a onClick={() => setChildren(children * 2)}>
						...{tree.children.size - children} more
					</a>
				</li>
			)}
		</ul>
	)
}

export function TagTreePage(p: {
	routeMatch: CWCRouteMatch
}): React.ReactElement {
	const [tag, setTag] = useState("")
	return (
		<Page title="Category Trees">
			Filter Tag:{" "}
			<input
				type="text"
				value={tag}
				onChange={(e) => setTag(e.currentTarget.value)}
			/>
			<ChooserWithChild
				routeMatch={p.routeMatch}
				child={(e) => <TagTree tagName={tag || undefined} {...e} />}
			/>
		</Page>
	)
}
@observer
export class TagTree extends React.Component<{
	events: SingleExtractedEvent[]
	tagName?: string
	chart?: boolean
}> {
	constructor(props: TagTree["props"]) {
		super(props)
		makeObservable(this)
	}
	@computed get tagTree(): ATree {
		const tree = rootTree<SingleExtractedEvent>()
		for (const event of this.props.events) {
			let added = false
			let toIter: [string, string[] | undefined][]
			if (this.props.tagName) {
				toIter = [
					[this.props.tagName, event.tags.map[this.props.tagName]],
				]
			} else {
				toIter = Object.entries(event.tags.map)
			}
			for (const [tagName, tagValues = []] of toIter) {
				for (const tagValue of tagValues) {
					addToTree(tree, [tagName, ...tagValue.split("/")], event)

					added = true
				}
			}
			if (this.props.tagName && !added) {
				addToTree(tree, [this.props.tagName, "[untagged]"], event)
			}
		}
		for (const c of tree.children) shortenTree(c[1])
		sortTree(tree)
		return tree
	}

	render(): React.ReactNode {
		const { chart = false } = this.props
		return (
			<div>
				{[...this.tagTree.children].map(([kind, tree]) => (
					<div key={kind}>
						<h3>
							{kind}{" "}
							{!chart && (
								<CategoryChartModalLink
									events={collect(tree)}
									deep={false}
									tag={kind}
								/>
							)}
						</h3>
						{chart && (
							<CategoryChart
								events={collect(tree)}
								deep={false}
								tag={kind + ":"}
							/>
						)}
						<ShowTreeChildren tree={tree} noSlash />
					</div>
				))}
			</div>
		)
	}
}
