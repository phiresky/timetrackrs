import _ from "lodash"
import { computed, makeObservable } from "mobx"
import { observer, useLocalObservable } from "mobx-react"
import * as React from "react"
import { useState } from "react"
import { Container } from "reactstrap"
import Modal from "react-modal"
import { CgDetailsMore } from "react-icons/cg"
import { SingleExtractedChunk, Timestamptz } from "../server"
import {
	Counter,
	DefaultMap,
	durationToString,
	getTagValues,
	totalDurationSeconds,
} from "../util"
import {
	CategoryChart,
	CategoryChartModal as CategoryChartModalLink,
} from "./CategoryChart"
import { ChooserWithChild, CWCRouteMatch } from "./ChooserWithChild"
import { Page } from "./Page"
import { Choices, Select } from "./Select"
import { Temporal } from "@js-temporal/polyfill"

interface Tree<T> {
	leaves: T[]
	children: Map<string, Tree<T>>
}

type ATree = Tree<ALeaf>

interface ALeaf {
	timeChunk: SingleExtractedChunk
	tagName: string
	tagValue: string
	duration: number
}

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

function collectRecurse(tree: ATree, add: (e: ALeaf) => void) {
	tree.leaves.forEach(add)
	for (const c of tree.children.values()) collectRecurse(c, add)
}
function collect(tree: ATree): ALeaf[] {
	const map = new Map<Timestamptz, ALeaf>()
	collectRecurse(tree, (e) => map.set(e.timeChunk.from, e))
	return [...map.values()]
}

function TotalDuration(props: { tree: ATree }) {
	return (
		<span>
			{durationToString(totalDurationSeconds(collect(props.tree)))}
		</span>
	)
}

const TreeLeaves: React.FC<{ leaves: ALeaf[] }> = observer(({ leaves }) => {
	const [children, setChildren] = React.useState(5)
	const store = useLocalObservable(() => {
		const totalCount = leaves.length
		const totalCounts = new Counter<string>()
		const valueCounter = new DefaultMap<string, Counter<string>>(
			() => new Counter(),
		)
		for (const l of leaves) {
			for (const [tagKey, value, _duration] of l.timeChunk.tags) {
				totalCounts.add(tagKey)
				valueCounter.get(tagKey).add(value)
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
					<li key={l.timeChunk.from.toString()}>
						{l.timeChunk.from
							.toZonedDateTimeISO(Temporal.Now.timeZone())
							.toLocaleString(undefined, { timeStyle: "medium" })}
						:
						<ul>
							{l.timeChunk.tags.map(([k, v, t]) => (
								<li key={k + v}>
									{k}: {v} ({(t / 1000).toFixed(1)} s)
								</li>
							))}
						</ul>
						{/*<Entry {...l} />*/}
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
			<TagTree
				timeChunks={leaves.map((l) => l.timeChunk)}
				tagName={store.choices.value.value}
			/>
		)
	return (
		<div>
			grouping by{" "}
			<Select<{ value: string; name: string }>
				target={store.choices}
				getValue={(e) => e.value}
				getName={(e) => e.name}
			/>
			{inner}
		</div>
	)
})
const ShowTree: React.FC<{
	tag: string
	tagValue: string
	tree: ATree
	noSlash?: boolean
}> = ({ tag, tagValue, tree, noSlash = false }) => {
	const [open, setOpen] = React.useState(false)

	const title = (noSlash ? "" : "/") + tagValue || "[empty]"

	return (
		<li key={tagValue}>
			<span className="clickable" onClick={() => setOpen(!open)}>
				{title} (<TotalDuration tree={tree} />){" "}
				<a href="detaaa">
					<DetailsButtonModal
						chunks={tree.leaves.map((m) => m.timeChunk)}
						tag={tag}
						tagValue={tagValue}
					/>
				</a>
			</span>

			{open && <ShowTreeChildren tag={tag} tree={tree} />}
			{open && tree.leaves.length > 0 && (
				<TreeLeaves leaves={tree.leaves} />
			)}
		</li>
	)
}
const DetailsButtonModal: React.FC<{
	chunks: SingleExtractedChunk[]
	tag: string
	tagValue: string
}> = ({ chunks }) => {
	const [open, setOpen] = useState(false)
	if (!open) {
		return (
			<a
				onClick={(e) => {
					e.preventDefault()
					setOpen(true)
				}}
			>
				<CgDetailsMore></CgDetailsMore>
			</a>
		)
	}
	const eventIds = chunks.flatMap((chunk) =>
		getTagValues(chunk.tags, "timetrackrs-raw-id").map(([v, _dur]) => v),
	)
	return (
		<Modal isOpen={true} onRequestClose={(_) => setOpen(false)}>
			Event ids: {eventIds.join("\n")}
		</Modal>
	)
}
const ShowTreeChildren: React.FC<{
	tree: ATree
	noSlash?: boolean
	tag: string
}> = ({ tree, noSlash, tag }) => {
	const [children, setChildren] = React.useState(5)
	return (
		<ul>
			{[...tree.children.entries()]
				.slice(0, children)
				.map(([tagValue, tree]) => (
					<ShowTree
						tag={tag}
						key={tagValue}
						tagValue={tagValue}
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
	return (
		<Page title="Category Trees">
			<Container fluid className="bg-gradient-info py-6">
				<Container>
					<ChooserWithChild
						routeMatch={p.routeMatch}
						chooseTag={false}
						child={(e) => <TagTree {...e} />}
					/>
				</Container>
			</Container>
		</Page>
	)
}

@observer
export class TagTree extends React.Component<{
	timeChunks: SingleExtractedChunk[]
	chart?: boolean
	tagName?: string
}> {
	constructor(props: TagTree["props"]) {
		super(props)
		makeObservable(this)
	}
	@computed get tagTree(): ATree {
		const tree = rootTree<ALeaf>()
		for (const timeChunk of this.props.timeChunks) {
			for (const [tagName, tagValue, duration] of timeChunk.tags) {
				if (this.props.tagName && this.props.tagName !== tagName)
					continue
				addToTree(tree, [tagName, ...tagValue.split("/")], {
					timeChunk,
					tagName,
					tagValue,
					duration,
				})
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
				{[...this.tagTree.children].map(([tag, tree]) => (
					<div key={tag}>
						<h3>
							{tag}{" "}
							{!chart && (
								<CategoryChartModalLink
									timeChunks={collect(tree).map(
										(t) => t.timeChunk,
									)}
									deep={false}
									tag={tag}
								/>
							)}
						</h3>
						{chart && (
							<CategoryChart
								timeChunks={collect(tree).map(
									(t) => t.timeChunk,
								)}
								deep={false}
								tag={tag + ":"}
							/>
						)}
						<ShowTreeChildren tag={tag} tree={tree} noSlash />
					</div>
				))}
			</div>
		)
	}
}
