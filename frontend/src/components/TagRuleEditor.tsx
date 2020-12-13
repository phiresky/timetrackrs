import { observer, useLocalObservable } from "mobx-react"
import { fromPromise } from "mobx-utils"
import * as React from "react"
import { TagRule, TagRuleGroup, TagRuleWithMeta } from "../server"
import { Page } from "./Page"
import * as api from "../api"
import { action, observable, runInAction } from "mobx"
import { generateId, intersperse } from "../util"
import { useState } from "react"
import { RegexEditor } from "./RegexEditor"
import { O_DIRECTORY } from "constants"
import AutosizeInput from "./AutosizeInput"
import { Choices, Select } from "./Select"

export function TagRuleEditorPage(): React.ReactElement {
	return (
		<Page>
			<TagRuleEditor />
		</Page>
	)
}

async function upload(g: TagRuleGroup) {
	await api.saveTagRules([g])
}

const TagRuleEditor: React.FC = observer(() => {
	const store = useLocalObservable(() => ({
		data: fromPromise(api.getTagRules().then((e) => observable(e))),
	}))

	return (
		<div className="centerbody">
			{store.data.case({
				rejected(v) {
					return <>Error loading: {String(v)}</>
				},
				pending() {
					return <>Loading...</>
				},
				fulfilled(v) {
					return (
						<>
							<button
								onClick={(_) =>
									v.unshift({
										global_id: generateId(),
										data: {
											version: "V1",
											data: {
												description: "",
												editable: true,
												enabled: true,
												name:
													prompt(
														"Group Name",
														"Untitled Group",
													) || "Untitled Group",
												rules: [],
											},
										},
									})
								}
							>
								Create New Group
							</button>
							{v.map((g) => (
								<TagRuleGroupEditor
									key={g.global_id}
									group={g}
									save={() => upload(g)}
								/>
							))}
						</>
					)
				},
			})}
		</div>
	)
})

const tagRulePrototypes: () => (TagRule | { type: "Add Rule" })[] = () => [
	{ type: "Add Rule" },
	{ type: "HasTag", tag: "...", new_tag: "..." },
	{ type: "ExactTagValue", tag: "...", value: "...", new_tag: "..." },
	{ type: "TagValuePrefix", tag: "...", prefix: "...", new_tag: "..." },
	{ type: "TagRegex", regexes: ["^...$"], new_tag: "..." },
]
const TagRuleGroupEditor: React.FC<{
	group: TagRuleGroup
	save: () => Promise<void>
}> = observer(({ group, save }) => {
	if (group.data.version !== "V1")
		throw Error("unexpected group data version")
	const g = group.data.data
	const [dirty, setDirty] = useState(false)
	return (
		<details className="rule-group">
			<summary>
				<h2>
					Group <em>{g.name}</em> {!g.editable && <>(Not editable)</>}
					{dirty && (
						<button
							onClick={async () => {
								await save()
								setDirty(false)
							}}
						>
							Save changes
						</button>
					)}
				</h2>
			</summary>
			<div className="rule-group-detail">
				Description: {g.description}
				<h3>Rules:</h3>
				{g.rules.map((r, i) => (
					<RuleEditor
						key={i}
						index={i}
						rule={r}
						editable={g.editable}
						dirty={() => setDirty(true)}
					/>
				))}
				{g.editable && (
					<Select
						getValue={(v) => v.type}
						getName={(v) => v.type}
						target={Choices(tagRulePrototypes())}
						onChange={(v) =>
							v.type !== "Add Rule" &&
							g.rules.push({ enabled: true, rule: { ...v } })
						}
					/>
				)}
			</div>
		</details>
	)
})

type RuleMoppies = { [T in TagRule["type"]]: TagRule & { type: T } }

function _InputWithTarget<K extends string>(p: {
	dirty: () => void
	target: { [k in K]: string }
	disabled: boolean
	k: K
}) {
	return (
		<AutosizeInput
			className="input-border"
			minWidth={50}
			disabled={p.disabled}
			value={p.target[p.k]}
			onChange={action((e: React.ChangeEvent<HTMLInputElement>) => {
				p.target[p.k] = e.currentTarget.value
				p.dirty()
			})}
		/>
	)
}
const InputWithTarget = observer(_InputWithTarget)

const ruleEditors: {
	[k in keyof RuleMoppies]: React.FC<{
		rule: RuleMoppies[k]
		editable: boolean
		dirty: () => void
	}>
} = {
	HasTag(p) {
		return (
			<div className="has-tag-rule">
				If tag{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="tag"
					dirty={p.dirty}
				/>{" "}
				exists
				<br />
				Then add new tag:{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="new_tag"
					dirty={p.dirty}
				/>
			</div>
		)
	},
	ExactTagValue(p) {
		return (
			<div className="exact-tag-value-rule">
				If tag{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="tag"
					dirty={p.dirty}
				/>{" "}
				has value{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="value"
					dirty={p.dirty}
				/>
				<br />
				Then add new tag:{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="new_tag"
					dirty={p.dirty}
				/>
			</div>
		)
	},
	TagValuePrefix(p) {
		return (
			<div className="tag-value-prefix-rule">
				If tag{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="tag"
					dirty={p.dirty}
				/>{" "}
				has prefix{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="prefix"
					dirty={p.dirty}
				/>
				<br />
				Then add new tag:{" "}
				<InputWithTarget
					disabled={!p.editable}
					target={p.rule}
					k="new_tag"
					dirty={p.dirty}
				/>
			</div>
		)
	},
	TagRegex(p) {
		return (
			<div className="tag-regex-rule">
				If{" "}
				{p.rule.regexes.length > 1
					? "all of the following regexes match"
					: "the following regex matches"}
				:{" "}
				{intersperse(
					p.rule.regexes.map((r, i) => (
						<RegexEditor
							key={i}
							editable={p.editable}
							value={r}
							onChange={(r) =>
								runInAction(() => {
									p.rule.regexes[i] = r
									p.dirty()
								})
							}
						/>
					)),
					(i) => (
						<React.Fragment key={`a${i}`}> and </React.Fragment>
					),
				)}{" "}
				<button
					onClick={() => {
						p.rule.regexes.push("^...$")
					}}
				>
					+
				</button>{" "}
				{p.rule.regexes.length > 1 && (
					<button
						onClick={() => {
							p.rule.regexes.pop()
							p.dirty()
						}}
					>
						-
					</button>
				)}
				<div>
					Then add new tag:{" "}
					<InputWithTarget
						disabled={!p.editable}
						target={p.rule}
						k="new_tag"
						dirty={p.dirty}
					/>
				</div>
			</div>
		)
	},
	InternalFetcher(p) {
		return <em>[internal fetcher {p.rule.fetcher_id}]</em>
	},
	ExternalFetcher(p) {
		return <em>[external caching fetcher {p.rule.fetcher_id}]</em>
	},
}
for (const [k, v] of Object.entries(ruleEditors) as [
	keyof typeof ruleEditors,
	typeof ruleEditors[keyof typeof ruleEditors],
][])
	ruleEditors[k] = observer(v) as React.FC<any>

const RuleEditor: React.FC<{
	index: number
	rule: TagRuleWithMeta
	editable: boolean
	dirty: () => void
}> = observer(({ dirty, index, rule, editable }) => {
	return (
		<div className="rule-editor">
			<h4>
				<label className="clickable">
					<input
						type="checkbox"
						checked={rule.enabled}
						onChange={(e) =>
							runInAction(() => {
								rule.enabled = e.currentTarget.checked
								dirty()
							})
						}
					/>{" "}
					Rule {index + 1}
					{rule.enabled ? "" : " (disabled)"}
				</label>
			</h4>
			{React.createElement(
				(ruleEditors[rule.rule.type] || (() => <p>UNK</p>)) as any,
				{
					rule: rule.rule,
					editable,
					dirty,
				},
			)}
		</div>
	)
})
