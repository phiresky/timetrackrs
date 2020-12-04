import _ from "lodash"
import { computed } from "mobx"
import { observer } from "mobx-react"
import * as React from "react"
import * as api from "../api"
import { durationToString, totalDuration } from "../util"
import { CategoryChart } from "./CategoryChart"
import { ChooserWithChild } from "./ChooserWithChild"
import { Entry } from "./Entry"
import { ModalLink } from "./ModalLink"
import { Page } from "./Page"

interface Tree<T> {
	leaves: T[]
	children: Map<string, Tree<T>>
}
type ATree = Tree<api.Activity>

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
		if (tree.children.size === 1) {
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
			v = -totalDuration(collect(t))
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

function collectRecurse(tree: ATree, add: (e: api.Activity) => void) {
	tree.leaves.forEach(add)
	for (const c of tree.children.values()) collectRecurse(c, add)
}
function collect(tree: ATree) {
	const map = new Map<string, api.Activity>()
	collectRecurse(tree, (e) => map.set(e.id, e))
	return [...map.values()]
}

function TotalDuration(props: { tree: ATree }) {
	return <span>{durationToString(totalDuration(collect(props.tree)))}</span>
}
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
	const [children, setChildren] = React.useState(5)

	const title = (noSlash ? "" : "/") + tag || "[empty]"

	return (
		<li key={tag}>
			<span className="clickable" onClick={() => setOpen(!open)}>
				{title} (<TotalDuration tree={tree} />)
			</span>

			{open && <ShowTreeChildren tree={tree} />}
			{open && tree.leaves.length > 0 && (
				<ul>
					{tree.leaves.slice(0, children).map((l) => (
						<li key={l.id}>
							<ModalLink to={`/single-event/${l.id}`}>
								<Entry {...l} />
							</ModalLink>
						</li>
					))}
					{tree.leaves.length > children && (
						<li key="more" className="clickable">
							<a
								className="clickable"
								onClick={() => setChildren(children * 2)}
							>
								...{tree.leaves.length - children} more
							</a>
						</li>
					)}
				</ul>
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

export function TagTreePage(): React.ReactElement {
	return (
		<Page title="Category Trees">
			<ChooserWithChild child={TagTree} />
		</Page>
	)
}
@observer
export class TagTree extends React.Component<{ events: api.Activity[] }> {
	constructor(props: TagTree["props"]) {
		super(props)
	}
	@computed get tagTree(): ATree {
		const tree = rootTree<api.Activity>()
		for (const event of this.props.events) {
			for (const tag of event.tags) {
				const inx = tag.indexOf(":")
				addToTree(
					tree,
					[tag.slice(0, inx), ...tag.slice(inx + 1).split("/")],
					event,
				)
			}
		}
		for (const c of tree.children) shortenTree(c[1])
		sortTree(tree)
		console.log(tree)
		return tree
	}

	render(): React.ReactNode {
		return (
			<div>
				{[...this.tagTree.children].map(([kind, tree]) => (
					<div key={kind}>
						<h3>{kind}</h3>
						<CategoryChart
							events={collect(tree)}
							deep={false}
							tagPrefix={kind + ":"}
						/>
						<ShowTreeChildren tree={tree} noSlash />
					</div>
				))}
			</div>
		)
	}
}
