import { differenceInDays } from "date-fns"
import * as d from "date-fns/esm"
import { startOfDay } from "date-fns/esm"
import { computed, makeObservable, observable } from "mobx"
import { observer } from "mobx-react"
import { Template } from "plotly.js"
import React from "react"
import Plotly from "react-plotly.js"
import { Card, Container } from "reactstrap"
import { SingleExtractedChunk, Timestamptz } from "../server"
import { DefaultMap, expectNever, NeatMap } from "../util"
import { ChooserWithChild, CWCRouteMatch } from "./ChooserWithChild"
import { Page } from "./Page"
import { Choices, Select } from "./Select"
import { SingleEventInfo } from "./SingleEventInfo"
import { Temporal, Intl, toTemporalInstant } from "@js-temporal/polyfill"

const dark = {
	data: {
		bar: [
			{
				error_x: { color: "#f2f5fa" },
				error_y: { color: "#f2f5fa" },
				marker: { line: { color: "rgb(17,17,17)", width: 0.5 } },
				type: "bar",
			},
		],
		barpolar: [
			{
				marker: { line: { color: "rgb(17,17,17)", width: 0.5 } },
				type: "barpolar",
			},
		],
		carpet: [
			{
				aaxis: {
					endlinecolor: "#A2B1C6",
					gridcolor: "#506784",
					linecolor: "#506784",
					minorgridcolor: "#506784",
					startlinecolor: "#A2B1C6",
				},
				baxis: {
					endlinecolor: "#A2B1C6",
					gridcolor: "#506784",
					linecolor: "#506784",
					minorgridcolor: "#506784",
					startlinecolor: "#A2B1C6",
				},
				type: "carpet",
			},
		],
		choropleth: [
			{ colorbar: { outlinewidth: 0, ticks: "" }, type: "choropleth" },
		],
		contour: [
			{
				colorbar: { outlinewidth: 0, ticks: "" },
				colorscale: [
					[0.0, "#0d0887"],
					[0.1111111111111111, "#46039f"],
					[0.2222222222222222, "#7201a8"],
					[0.3333333333333333, "#9c179e"],
					[0.4444444444444444, "#bd3786"],
					[0.5555555555555556, "#d8576b"],
					[0.6666666666666666, "#ed7953"],
					[0.7777777777777778, "#fb9f3a"],
					[0.8888888888888888, "#fdca26"],
					[1.0, "#f0f921"],
				],
				type: "contour",
			},
		],
		contourcarpet: [
			{ colorbar: { outlinewidth: 0, ticks: "" }, type: "contourcarpet" },
		],
		heatmap: [
			{
				colorbar: { outlinewidth: 0, ticks: "" },
				colorscale: [
					[0.0, "#0d0887"],
					[0.1111111111111111, "#46039f"],
					[0.2222222222222222, "#7201a8"],
					[0.3333333333333333, "#9c179e"],
					[0.4444444444444444, "#bd3786"],
					[0.5555555555555556, "#d8576b"],
					[0.6666666666666666, "#ed7953"],
					[0.7777777777777778, "#fb9f3a"],
					[0.8888888888888888, "#fdca26"],
					[1.0, "#f0f921"],
				],
				type: "heatmap",
			},
		],
		heatmapgl: [
			{
				colorbar: { outlinewidth: 0, ticks: "" },
				colorscale: [
					[0.0, "#0d0887"],
					[0.1111111111111111, "#46039f"],
					[0.2222222222222222, "#7201a8"],
					[0.3333333333333333, "#9c179e"],
					[0.4444444444444444, "#bd3786"],
					[0.5555555555555556, "#d8576b"],
					[0.6666666666666666, "#ed7953"],
					[0.7777777777777778, "#fb9f3a"],
					[0.8888888888888888, "#fdca26"],
					[1.0, "#f0f921"],
				],
				type: "heatmapgl",
			},
		],
		histogram: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "histogram",
			},
		],
		histogram2d: [
			{
				colorbar: { outlinewidth: 0, ticks: "" },
				colorscale: [
					[0.0, "#0d0887"],
					[0.1111111111111111, "#46039f"],
					[0.2222222222222222, "#7201a8"],
					[0.3333333333333333, "#9c179e"],
					[0.4444444444444444, "#bd3786"],
					[0.5555555555555556, "#d8576b"],
					[0.6666666666666666, "#ed7953"],
					[0.7777777777777778, "#fb9f3a"],
					[0.8888888888888888, "#fdca26"],
					[1.0, "#f0f921"],
				],
				type: "histogram2d",
			},
		],
		histogram2dcontour: [
			{
				colorbar: { outlinewidth: 0, ticks: "" },
				colorscale: [
					[0.0, "#0d0887"],
					[0.1111111111111111, "#46039f"],
					[0.2222222222222222, "#7201a8"],
					[0.3333333333333333, "#9c179e"],
					[0.4444444444444444, "#bd3786"],
					[0.5555555555555556, "#d8576b"],
					[0.6666666666666666, "#ed7953"],
					[0.7777777777777778, "#fb9f3a"],
					[0.8888888888888888, "#fdca26"],
					[1.0, "#f0f921"],
				],
				type: "histogram2dcontour",
			},
		],
		mesh3d: [{ colorbar: { outlinewidth: 0, ticks: "" }, type: "mesh3d" }],
		parcoords: [
			{
				line: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "parcoords",
			},
		],
		pie: [{ automargin: true, type: "pie" }],
		scatter: [{ marker: { line: { color: "#283442" } }, type: "scatter" }],
		scatter3d: [
			{
				line: { colorbar: { outlinewidth: 0, ticks: "" } },
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scatter3d",
			},
		],
		scattercarpet: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scattercarpet",
			},
		],
		scattergeo: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scattergeo",
			},
		],
		scattergl: [
			{ marker: { line: { color: "#283442" } }, type: "scattergl" },
		],
		scattermapbox: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scattermapbox",
			},
		],
		scatterpolar: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scatterpolar",
			},
		],
		scatterpolargl: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scatterpolargl",
			},
		],
		scatterternary: [
			{
				marker: { colorbar: { outlinewidth: 0, ticks: "" } },
				type: "scatterternary",
			},
		],
		surface: [
			{
				colorbar: { outlinewidth: 0, ticks: "" },
				colorscale: [
					[0.0, "#0d0887"],
					[0.1111111111111111, "#46039f"],
					[0.2222222222222222, "#7201a8"],
					[0.3333333333333333, "#9c179e"],
					[0.4444444444444444, "#bd3786"],
					[0.5555555555555556, "#d8576b"],
					[0.6666666666666666, "#ed7953"],
					[0.7777777777777778, "#fb9f3a"],
					[0.8888888888888888, "#fdca26"],
					[1.0, "#f0f921"],
				],
				type: "surface",
			},
		],
		table: [
			{
				cells: {
					fill: { color: "#506784" },
					line: { color: "rgb(17,17,17)" },
				},
				header: {
					fill: { color: "#2a3f5f" },
					line: { color: "rgb(17,17,17)" },
				},
				type: "table",
			},
		],
	},
	layout: {
		annotationdefaults: {
			arrowcolor: "#f2f5fa",
			arrowhead: 0,
			arrowwidth: 1,
		},
		autotypenumbers: "strict",
		coloraxis: { colorbar: { outlinewidth: 0, ticks: "" } },
		colorscale: {
			diverging: [
				[0, "#8e0152"],
				[0.1, "#c51b7d"],
				[0.2, "#de77ae"],
				[0.3, "#f1b6da"],
				[0.4, "#fde0ef"],
				[0.5, "#f7f7f7"],
				[0.6, "#e6f5d0"],
				[0.7, "#b8e186"],
				[0.8, "#7fbc41"],
				[0.9, "#4d9221"],
				[1, "#276419"],
			],
			sequential: [
				[0.0, "#0d0887"],
				[0.1111111111111111, "#46039f"],
				[0.2222222222222222, "#7201a8"],
				[0.3333333333333333, "#9c179e"],
				[0.4444444444444444, "#bd3786"],
				[0.5555555555555556, "#d8576b"],
				[0.6666666666666666, "#ed7953"],
				[0.7777777777777778, "#fb9f3a"],
				[0.8888888888888888, "#fdca26"],
				[1.0, "#f0f921"],
			],
			sequentialminus: [
				[0.0, "#0d0887"],
				[0.1111111111111111, "#46039f"],
				[0.2222222222222222, "#7201a8"],
				[0.3333333333333333, "#9c179e"],
				[0.4444444444444444, "#bd3786"],
				[0.5555555555555556, "#d8576b"],
				[0.6666666666666666, "#ed7953"],
				[0.7777777777777778, "#fb9f3a"],
				[0.8888888888888888, "#fdca26"],
				[1.0, "#f0f921"],
			],
		},
		colorway: [
			"#636efa",
			"#EF553B",
			"#00cc96",
			"#ab63fa",
			"#FFA15A",
			"#19d3f3",
			"#FF6692",
			"#B6E880",
			"#FF97FF",
			"#FECB52",
		],
		font: { color: "#f2f5fa" },
		geo: {
			bgcolor: "rgb(17,17,17)",
			lakecolor: "rgb(17,17,17)",
			landcolor: "rgb(17,17,17)",
			showlakes: true,
			showland: true,
			subunitcolor: "#506784",
		},
		hoverlabel: { align: "left" },
		hovermode: "closest",
		mapbox: { style: "dark" },
		paper_bgcolor: "rgb(17,17,17)",
		plot_bgcolor: "rgb(17,17,17)",
		polar: {
			angularaxis: {
				gridcolor: "#506784",
				linecolor: "#506784",
				ticks: "",
			},
			bgcolor: "rgb(17,17,17)",
			radialaxis: {
				gridcolor: "#506784",
				linecolor: "#506784",
				ticks: "",
			},
		},
		scene: {
			xaxis: {
				backgroundcolor: "rgb(17,17,17)",
				gridcolor: "#506784",
				gridwidth: 2,
				linecolor: "#506784",
				showbackground: true,
				ticks: "",
				zerolinecolor: "#C8D4E3",
			},
			yaxis: {
				backgroundcolor: "rgb(17,17,17)",
				gridcolor: "#506784",
				gridwidth: 2,
				linecolor: "#506784",
				showbackground: true,
				ticks: "",
				zerolinecolor: "#C8D4E3",
			},
			zaxis: {
				backgroundcolor: "rgb(17,17,17)",
				gridcolor: "#506784",
				gridwidth: 2,
				linecolor: "#506784",
				showbackground: true,
				ticks: "",
				zerolinecolor: "#C8D4E3",
			},
		},
		shapedefaults: { line: { color: "#f2f5fa" } },
		sliderdefaults: {
			bgcolor: "#C8D4E3",
			bordercolor: "rgb(17,17,17)",
			borderwidth: 1,
			tickwidth: 0,
		},
		ternary: {
			aaxis: { gridcolor: "#506784", linecolor: "#506784", ticks: "" },
			baxis: { gridcolor: "#506784", linecolor: "#506784", ticks: "" },
			bgcolor: "rgb(17,17,17)",
			caxis: { gridcolor: "#506784", linecolor: "#506784", ticks: "" },
		},
		title: { x: 0.05 },
		updatemenudefaults: { bgcolor: "#506784", borderwidth: 0 },
		xaxis: {
			automargin: true,
			gridcolor: "#283442",
			linecolor: "#506784",
			ticks: "",
			title: { standoff: 15 },
			zerolinecolor: "#283442",
			zerolinewidth: 2,
		},
		yaxis: {
			automargin: true,
			gridcolor: "#283442",
			linecolor: "#506784",
			ticks: "",
			title: { standoff: 15 },
			zerolinecolor: "#283442",
			zerolinewidth: 2,
		},
	},
}

export function PlotPage(p: { routeMatch: CWCRouteMatch }): React.ReactElement {
	return (
		<Page title="Plot">
			<Container fluid className="bg-gradient-info py-6">
				<Card>
					<ChooserWithChild
						chooseTag={true}
						routeMatch={p.routeMatch}
						child={Plot}
						containerClass="mx-auto"
					/>
				</Card>
			</Container>
		</Page>
	)
}

type Aggregator = { mapper: (d: Date) => Date; name: string; visible: boolean }

function getValue(deep: boolean, value: string) {
	if (deep) return value
	else return value.split("/")[0]
}
function aggregateData({
	aggregator,
	binSize,
	events,
	tag,
	deep,
}: {
	aggregator: (d: Date) => Date
	binSize: number
	events: SingleExtractedChunk[]
	tag: string
	deep: boolean
}) {
	// const { days } = this.dayInfo

	type TagValue = string
	type Bucket = string
	type absolute_duration = number

	const outData = new DefaultMap<
		TagValue,
		NeatMap<Bucket, absolute_duration>
	>(() => new NeatMap<string, number>())
	const totalDuration = new NeatMap<Bucket, number>()

	for (const timechunk of events) {
		const bucketD = aggregator(
			new Date(timechunk.from - (timechunk.from % binSize)),
		)
		// to local time text
		const bucket = toTemporalInstant
			.call(bucketD)

			.toZonedDateTimeISO(Temporal.Now.timeZone())
			.toPlainDateTime()
			.toString()
		for (const [mtag, _value, duration] of timechunk.tags) {
			if (tag !== mtag) continue
			const value = getValue(deep, _value)
			outData.get(value).addDelta(bucket, duration)
			totalDuration.addDelta(bucket, duration)
		}
	}

	//const { firstDay, lastDay, days: aggDays } = this.getDayInfo(data)
	return { outData, totalDuration }
}
const colorScale = [
	"#636EFA",
	"#EF553B",
	"#00CC96",
	"#AB63FA",
	"#FFA15A",
	"#19D3F3",
	"#FF6692",
	"#B6E880",
	"#FF97FF",
	"#FECB52",
]

const fixedColors: Record<string, string> = {
	Productivity: colorScale[0],
	Entertainment: colorScale[1],
	Communication: colorScale[2],
}
const usedColors = new Set(Object.values(fixedColors))
const unusedColors = colorScale.filter((c) => !usedColors.has(c))
function getColors(labels: string[]): string[] {
	const availableColors = new Set(unusedColors)
	return labels.map((label) => {
		if (label in fixedColors) {
			return fixedColors[label]
		} else {
			const c = availableColors[Symbol.iterator]().next()
			if (!c.done) {
				availableColors.delete(c.value)
				return c.value
			} else {
				return colorScale[(Math.random() * colorScale.length) | 0]
			}
		}
	})
}
export class InnerPlot extends React.Component<{
	events: SingleExtractedChunk[]
	chartType?: "stacked-bar" | "pie"
	tag: string
	deep: boolean
	scaleTo100: boolean
	aggregator: Aggregator
	binSize: number
	dark: boolean
}> {
	@computed get data(): Plotly.Data[] {
		const binSize = this.props.binSize
		const { outData, totalDuration } = aggregateData({
			aggregator: this.props.aggregator.mapper,
			binSize,
			events: this.props.events,
			tag: this.props.tag,
			deep: this.props.deep,
		})
		const { chartType = "stacked-bar" } = this.props
		const labels = [...outData.keys()]
		if (chartType === "pie") {
			return [
				{
					type: "pie",
					labels,
					values: [...outData.values()].map(
						(es) => [...es.values()][0],
					),
					marker: {
						colors: getColors(labels),
					},
				},
			]
		} else if (chartType === "stacked-bar") {
			const colors = new Map(
				getColors(labels).map((e, i) => [labels[i], e]),
			)
			const data: Plotly.Data[] = [...outData].map(([key, es]) => {
				//

				const aggFactor = 1 // aggDays / days
				const es2 = [...es]
				const scaleFactor = (bucket: string) =>
					this.props.scaleTo100
						? 1 / (totalDuration.get(bucket) || 1)
						: aggFactor / binSize

				return {
					/*xaxis: {
						tick0: firstDay,
					},*/
					x: es2.map((x) => x[0]),
					y: es2.map((x) => x[1] * scaleFactor(x[0])),
					type: "bar",
					name: key,
					marker: {
						color: colors.get(key),
					},
				}
			})
			console.log("data", data)
			return data
		} else {
			throw Error(`unknown chart type ${expectNever<string>(chartType)}`)
		}
	}

	render() {
		const agg = this.props.aggregator
		return (
			<Plotly
				className="maxwidth"
				data={this.data}
				layout={{
					height: 400,
					margin: {
						// reduce margins because of missing title
						l: 0,
						r: 0,
						b: 0,
						t: 0,
						pad: 4,
					},
					plot_bgcolor: "#0000",
					paper_bgcolor: "#0000",
					template: this.props.dark
						? (dark as unknown as Template)
						: undefined,
					autosize: true,
					// title: null, // `Time spent by ${this.props.tag}`,
					barmode: "stack",
					bargap: 0,
					legend: { orientation: "h" },

					yaxis: {
						// https://github.com/d3/d3-format/blob/master/README.md#locale_format
						title: "Spent time (%)",
						tickformat: "percent",
						hoverformat: ".0%",
					},
					xaxis: {
						// https://github.com/d3/d3-time-format/blob/master/README.md
						tickformat:
							agg.name == "None"
								? undefined
								: agg.name === "Daily"
								? "%H:%M:%S"
								: agg.name === "Weekly"
								? "%A %H:%M"
								: undefined,
					},
				}}
			/>
		)
	}
}
@observer
export class Plot extends React.Component<{
	timeChunks: SingleExtractedChunk[]
	tag: string
}> {
	r = React.createRef<HTMLDivElement>()

	plot: Plotly.PlotlyHTMLElement | null = null

	@observable deep = false
	@observable scaleTo100 = false
	@computed get binSizes() {
		let totalDur: number
		if (this.aggregators.value.name === "Daily")
			totalDur = 1000 * 60 * 60 * 24
		else if (this.aggregators.value.name === "Weekly")
			totalDur = 1000 * 60 * 60 * 24 * 7
		else {
			totalDur =
				this.dayInfo.last.getTime() - this.dayInfo.first.getTime()
		}
		console.log("total dur: ", totalDur / 1000 / 60 / 60 / 24)
		const allChoices = [
			{ value: 1000 * 60 * 5, name: "5 minutes" },
			{ value: 1000 * 60 * 20, name: "20 minutes" },
			{ value: 1000 * 60 * 60, name: "Hourly" },
			{ value: 1000 * 60 * 60 * 4, name: "4 h" },
			{ value: 1000 * 60 * 60 * 24, name: "Daily" },
		]
		const filteredChoices = allChoices.filter(
			(c) => c.value > totalDur / 1000 && c.value < totalDur / 4,
		)
		if (filteredChoices.length === 0) return observable(Choices(allChoices))
		return observable(Choices(filteredChoices))
	}
	@computed get aggregators() {
		const allChoices = [
			{ mapper: (date: Date) => date, name: "None", visible: true },
			{
				mapper: (date: Date) => {
					const duration = d.intervalToDuration({
						start: d.startOfDay(date),
						end: date,
					})
					const today = d.startOfDay(new Date("2021-01-01"))
					return d.add(today, duration)
				},
				name: "Daily",
				visible: this.dayInfo.days > 2,
			},
			{
				mapper: (date: Date) => {
					const duration = d.intervalToDuration({
						start: d.startOfWeek(date),
						end: date,
					})
					const today = d.startOfWeek(new Date())
					return d.add(today, duration)
				},
				name: "Weekly",
				visible: this.dayInfo.days > 14,
			},
		]
		const filteredChoices = allChoices.filter((c) => c.visible)
		if (filteredChoices.length === 0) return observable(Choices(allChoices))
		return observable(Choices(filteredChoices))
	}

	constructor(p: Plot["props"]) {
		super(p)
		makeObservable(this)
	}
	/*@computed get currentBinSize() {
		const v = this.binSize.value
		if (v.value <= 1000 * 60 && this.dayInfo.days > 1) {
			return this.binSize.choices.find((f) => f.value > 1000 * 60)!
		}
		return v
	}*/

	@computed get dayInfo() {
		return this.getDayInfo(this.props.timeChunks)
	}

	getDayInfo(e: { from: Timestamptz; to_exclusive: Timestamptz }[]) {
		if (e.length === 0) {
			return {
				firstDay: new Date(),
				lastDay: new Date(),
				days: 0,
				first: new Date(),
				last: new Date(),
				durMs: 0,
			}
		}
		const first = new Date(e[0].from)
		const last = new Date(+e[e.length - 1].to_exclusive - 1)
		const firstDay = startOfDay(first)
		const lastDay = startOfDay(last)
		const days = differenceInDays(lastDay, firstDay) + 1
		const durMs = last.getTime() - first.getTime()
		console.log(first, last)
		return { firstDay, lastDay, days, first, last, durMs }
	}

	render(): React.ReactElement {
		return (
			<div>
				Precision:{" "}
				<Select
					target={this.binSizes}
					getName={(e) => e.name}
					getValue={(e) => String(e.value)}
				/>{" "}
				{this.aggregators.choices.length > 1 && (
					<>
						Aggregation:{" "}
						<Select
							target={this.aggregators}
							getName={(e) => e.name}
							getValue={(e) => String(e.name)}
						/>{" "}
					</>
				)}
				<label>
					Deep{" "}
					<input
						type="checkbox"
						checked={this.deep}
						onChange={(e) => (this.deep = e.currentTarget.checked)}
					/>
				</label>
				<label>
					Relative{" "}
					<input
						type="checkbox"
						checked={this.scaleTo100}
						onChange={(e) =>
							(this.scaleTo100 = e.currentTarget.checked)
						}
					/>
				</label>
				<InnerPlot
					events={this.props.timeChunks}
					dark={false}
					tag={this.props.tag}
					deep={this.deep}
					aggregator={this.aggregators.value}
					binSize={this.binSizes.value.value}
					scaleTo100={this.scaleTo100}
				/>
			</div>
		)
	}
}
