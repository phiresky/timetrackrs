import { computed } from "mobx"
import React from "react"
import Plot from "react-plotly.js"
import { DefaultMap, getTagValues } from "../util"
import { ModalLink } from "./ModalLink"
import { AiOutlineBarChart } from "react-icons/ai"
import { routes } from "../routes"
import { SingleExtractedChunk } from "../server"

type CategoryChartProps = {
	timeChunks: SingleExtractedChunk[]
	tag: string
	deep: boolean
}

export function CategoryChartModal(p: CategoryChartProps): React.ReactElement {
	return (
		<ModalLink
			route={routes.categoryChart}
			args={{ tagName: p.tag }}
			query={{}}
		>
			<AiOutlineBarChart />
		</ModalLink>
	)
}
export class CategoryChart extends React.Component<CategoryChartProps> {
	@computed get data(): { x: string[]; y: number[] } {
		const tag = this.props.tag
		const groups = new DefaultMap<string, number>(() => 0)
		for (const timeChunk of this.props.timeChunks) {
			for (const [val, dur] of getTagValues(timeChunk.tags, tag)) {
				groups.addDelta(val, dur)
			}
		}
		const x = [...groups.keys()]
		const y = [...groups.values()].map((s) => s / 60 / 60)
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
						title: `Time spent per ${this.props.tag}`,
					}}
				/>
			</div>
		)
	}
}
