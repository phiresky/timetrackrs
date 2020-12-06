import { observer, useLocalStore } from "mobx-react"
import { fromPromise } from "mobx-utils"
import * as React from "react"
import { TagRuleGroup } from "../server"
import { Page } from "./Page"

export function TagRuleEditorPage(): React.ReactElement {
	return (
		<Page>
			<TagRuleEditor />
		</Page>
	)
}

/*const TagRuleEditor: React.FC = () => {
	const data = useLocalStore(() => {
		data: fromPromise()
	})
}
*/
