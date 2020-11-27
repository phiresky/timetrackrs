import * as React from "react"
import * as dfn from "date-fns"
import { computed, observable } from "mobx"
import { fromPromise, IPromiseBasedObservable } from "mobx-utils"
import * as api from "../api"
import { observer } from "mobx-react"
import { Link } from "react-router-dom"
import { ModalLink } from "./ModalLink"
import { Entry } from "./Entry"
import { durationToString, totalDuration } from "../util"
import { map } from "lodash"

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

	const title = (noSlash ? "" : "/") + tag || "[empty]"

	return (
		<li key={tag}>
			<span onClick={() => setOpen(!open)}>
				{title} (<TotalDuration tree={tree} />)
			</span>

			{open && <ShowTreeChildren tree={tree} />}
			{open && tree.leaves.length > 0 && (
				<ul>
					{tree.leaves.map((l) => (
						<li key={l.id}>
							<ModalLink to={`/single-event/${l.id}`}>
								<Entry {...l} />
							</ModalLink>
						</li>
					))}
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
	return (
		<ul>
			{[...tree.children.entries()].map(([tag, tree]) => (
				<ShowTree key={tag} tag={tag} tree={tree} noSlash={noSlash} />
			))}
		</ul>
	)
}

@observer
export class TagTree extends React.Component {
	@observable
	startTime: Date = dfn.subDays(new Date(), 10)
	@observable
	endTime: Date = new Date()

	@computed get data(): IPromiseBasedObservable<api.Activity[]> {
		return fromPromise(
			api.getTimeRange({ before: new Date(), limit: 10000 }),
		)
	}

	@computed get tagTree(): null | ATree {
		if (this.data.state !== "fulfilled") return null
		const events = this.data.value
		const tree = rootTree<api.Activity>()
		for (const event of events) {
			for (const tag of event.data.tags) {
				const inx = tag.indexOf(":")
				addToTree(
					tree,
					[tag.slice(0, inx), ...tag.slice(inx + 1).split("/")],
					event,
				)
			}
		}
		for (const c of tree.children) shortenTree(c[1])
		console.log(tree)
		return tree
	}

	render(): React.ReactNode {
		console.log(this.data.value)
		return (
			<div className="container">
				<div className="header">Tag Tree</div>

				<div>
					Events:{" "}
					{this.data.case({
						fulfilled: (v) => v.length.toString(),
						pending: () => "loading",
						rejected: (e) => {
							console.error("o", e)
							return String(e)
						},
					})}
				</div>
				{this.tagTree && (
					<>
						<h2>Category Trees</h2>
						{[...this.tagTree.children].map(([kind, tree]) => (
							<div key={kind}>
								<h3>{kind}</h3>
								<ShowTreeChildren tree={tree} noSlash />
							</div>
						))}
					</>
				)}
			</div>
		)
	}
}
