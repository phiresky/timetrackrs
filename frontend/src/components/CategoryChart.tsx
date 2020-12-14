import { computed } from "mobx"
import React from "react"
import Plot from "react-plotly.js"
import { DefaultMap, KeyedSet, totalDuration } from "../util"
import { ModalLink } from "./ModalLink"
import { AiOutlineBarChart } from "react-icons/ai"
import { SingleExtractedEvent } from "../server"

type CategoryChartProps = {
	events: SingleExtractedEvent[]
	tagName: string
	deep: boolean
}

export function CategoryChartModal(p: CategoryChartProps): React.ReactElement {
	return (
		<ModalLink to={`/category-chart/${p.tagName}`}>
			<AiOutlineBarChart />
		</ModalLink>
	)
}
export class CategoryChart extends React.Component<CategoryChartProps> {
	@computed get data() {
		const prefix = this.props.tagName
		const groups = new DefaultMap<string, KeyedSet<SingleExtractedEvent>>(
			() => new KeyedSet((e) => e.id),
		)
		for (const event of this.props.events) {
			for (let cat of event.tags.map[prefix] || []) {
				if (!this.props.deep) cat = cat.split("/")[0]
				groups.get(cat).add(event)
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
						title: `Time spent per ${this.props.tagName}`,
					}}
				/>
			</div>
		)
	}
}
