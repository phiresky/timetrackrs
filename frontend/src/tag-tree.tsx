import * as React from "react"
import * as dfn from "date-fns"
import { computed, observable } from "mobx"
import { fromPromise, IPromiseBasedObservable } from "mobx-utils"
import * as api from "./api"
import { observer } from "mobx-react"

interface Tree {
	children: Map<string, Tree>
}
function rootTree(): Tree {
	return { children: new Map<string, Tree>() }
}

function addToTree(t: Tree, path: string[]) {
	if (path.length === 0) return
	const [head, ...tail] = path
	let seg = t.children.get(head)
	if (!seg) {
		seg = rootTree()
		t.children.set(head, seg)
	}
	addToTree(seg, tail)
}

function shortenTree(t: Tree) {
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

function ShowTree({ tag, tree }: { tag: string; tree: Tree }) {
	const [open, setOpen] = React.useState(false)

	return (
		<li key={tag}>
			<span onClick={() => setOpen(!open)}>{tag || "[empty]"}</span>
			{open && (
				<ul>
					{[...tree.children.entries()].map(([tag, tree]) => (
						<ShowTree tag={tag} tree={tree} />
					))}
				</ul>
			)}
		</li>
	)
}

@observer
export class TagTree extends React.Component {
	@observable
	startTime: Date = dfn.subDays(new Date(), 1)
	@observable
	endTime: Date = new Date()

	@computed get data(): IPromiseBasedObservable<api.Activity[]> {
		return fromPromise(
			api.getTimeRange({ after: this.startTime, limit: 10000 }),
		)
	}

	@computed get tagTree(): null | Tree {
		if (this.data.state !== "fulfilled") return null
		const events = this.data.value
		const tree = rootTree()
		for (const event of events) {
			for (const tag of event.data.tags) {
				addToTree(tree, tag.split("/"))
			}
		}
		shortenTree(tree)
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
						Tree: <ShowTree tag="ROOT" tree={this.tagTree} />
					</>
				)}
			</div>
		)
	}
}
