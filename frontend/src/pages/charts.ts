/*!

=========================================================
* Argon Dashboard React - v1.2.0
=========================================================

* Product Page: https://www.creative-tim.com/product/argon-dashboard-react
* Copyright 2021 Creative Tim (https://www.creative-tim.com)
* Licensed under MIT (https://github.com/creativetimofficial/argon-dashboard-react/blob/master/LICENSE.md)

* Coded by Creative Tim

=========================================================

* The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

*/
import { Chart } from "chart.js"
//
// Chart extension for making the bars rounded
// Code from: https://codepen.io/jedtrow/full/ygRYgo
//

Chart.elements.Rectangle.prototype.draw = function () {
	const ctx = this._chart.ctx
	const vm = this._view
	var left, right, top, bottom, signX, signY, borderSkipped, radius
	let borderWidth = vm.borderWidth
	// Set Radius Here
	// If radius is large enough to cause drawing errors a max radius is imposed
	const cornerRadius = 6

	if (!vm.horizontal) {
		// bar
		left = vm.x - vm.width / 2
		right = vm.x + vm.width / 2
		top = vm.y
		bottom = vm.base
		signX = 1
		signY = bottom > top ? 1 : -1
		borderSkipped = vm.borderSkipped || "bottom"
	} else {
		// horizontal bar
		left = vm.base
		right = vm.x
		top = vm.y - vm.height / 2
		bottom = vm.y + vm.height / 2
		signX = right > left ? 1 : -1
		signY = 1
		borderSkipped = vm.borderSkipped || "left"
	}

	// Canvas doesn't allow us to stroke inside the width so we can
	// adjust the sizes to fit if we're setting a stroke on the line
	if (borderWidth) {
		// borderWidth shold be less than bar width and bar height.
		const barSize = Math.min(Math.abs(left - right), Math.abs(top - bottom))
		borderWidth = borderWidth > barSize ? barSize : borderWidth
		const halfStroke = borderWidth / 2
		// Adjust borderWidth when bar top position is near vm.base(zero).
		const borderLeft =
			left + (borderSkipped !== "left" ? halfStroke * signX : 0)
		const borderRight =
			right + (borderSkipped !== "right" ? -halfStroke * signX : 0)
		const borderTop =
			top + (borderSkipped !== "top" ? halfStroke * signY : 0)
		const borderBottom =
			bottom + (borderSkipped !== "bottom" ? -halfStroke * signY : 0)
		// not become a vertical line?
		if (borderLeft !== borderRight) {
			top = borderTop
			bottom = borderBottom
		}
		// not become a horizontal line?
		if (borderTop !== borderBottom) {
			left = borderLeft
			right = borderRight
		}
	}

	ctx.beginPath()
	ctx.fillStyle = vm.backgroundColor
	ctx.strokeStyle = vm.borderColor
	ctx.lineWidth = borderWidth

	// Corner points, from bottom-left to bottom-right clockwise
	// | 1 2 |
	// | 0 3 |
	const corners = [
		[left, bottom],
		[left, top],
		[right, top],
		[right, bottom],
	]

	// Find first (starting) corner with fallback to 'bottom'
	const borders = ["bottom", "left", "top", "right"]
	let startCorner = borders.indexOf(borderSkipped, 0)
	if (startCorner === -1) {
		startCorner = 0
	}

	function cornerAt(index) {
		return corners[(startCorner + index) % 4]
	}

	// Draw rectangle from 'startCorner'
	let corner = cornerAt(0)
	ctx.moveTo(corner[0], corner[1])

	for (let i = 1; i < 4; i++) {
		corner = cornerAt(i)
		let nextCornerId = i + 1
		if (nextCornerId === 4) {
			nextCornerId = 0
		}

		// let nextCorner = cornerAt(nextCornerId);

		const width = corners[2][0] - corners[1][0]
		const height = corners[0][1] - corners[1][1]
		const x = corners[1][0]
		const y = corners[1][1]
		// eslint-disable-next-line
		var radius = cornerRadius

		// Fix radius being too large
		if (radius > height / 2) {
			radius = height / 2
		}
		if (radius > width / 2) {
			radius = width / 2
		}

		ctx.moveTo(x + radius, y)
		ctx.lineTo(x + width - radius, y)
		ctx.quadraticCurveTo(x + width, y, x + width, y + radius)
		ctx.lineTo(x + width, y + height - radius)
		ctx.quadraticCurveTo(
			x + width,
			y + height,
			x + width - radius,
			y + height,
		)
		ctx.lineTo(x + radius, y + height)
		ctx.quadraticCurveTo(x, y + height, x, y + height - radius)
		ctx.lineTo(x, y + radius)
		ctx.quadraticCurveTo(x, y, x + radius, y)
	}

	ctx.fill()
	if (borderWidth) {
		ctx.stroke()
	}
}

const mode = "light" //(themeMode) ? themeMode : 'light';
const fonts = {
	base: "Open Sans",
}

// Colors
const colors = {
	gray: {
		100: "#f6f9fc",
		200: "#e9ecef",
		300: "#dee2e6",
		400: "#ced4da",
		500: "#adb5bd",
		600: "#8898aa",
		700: "#525f7f",
		800: "#32325d",
		900: "#212529",
	},
	theme: {
		default: "#172b4d",
		primary: "#5e72e4",
		secondary: "#f4f5f7",
		info: "#11cdef",
		success: "#2dce89",
		danger: "#f5365c",
		warning: "#fb6340",
	},
	black: "#12263F",
	white: "#FFFFFF",
	transparent: "transparent",
}

// Methods

// Chart.js global options
export function chartOptions() {
	// Options
	const options = {
		defaults: {
			global: {
				responsive: true,
				maintainAspectRatio: false,
				defaultColor:
					mode === "dark" ? colors.gray[700] : colors.gray[600],
				defaultFontColor:
					mode === "dark" ? colors.gray[700] : colors.gray[600],
				defaultFontFamily: fonts.base,
				defaultFontSize: 13,
				layout: {
					padding: 0,
				},
				legend: {
					display: false,
					position: "bottom",
					labels: {
						usePointStyle: true,
						padding: 16,
					},
				},
				elements: {
					point: {
						radius: 0,
						backgroundColor: colors.theme["primary"],
					},
					line: {
						tension: 0.4,
						borderWidth: 4,
						borderColor: colors.theme["primary"],
						backgroundColor: colors.transparent,
						borderCapStyle: "rounded",
					},
					rectangle: {
						backgroundColor: colors.theme["warning"],
					},
					arc: {
						backgroundColor: colors.theme["primary"],
						borderColor:
							mode === "dark" ? colors.gray[800] : colors.white,
						borderWidth: 4,
					},
				},
				tooltips: {
					enabled: true,
					mode: "index",
					intersect: false,
				},
			},
			doughnut: {
				cutoutPercentage: 83,
				legendCallback: function (chart) {
					const data = chart.data
					let content = ""

					data.labels.forEach(function (label, index) {
						const bgColor = data.datasets[0].backgroundColor[index]

						content += '<span class="chart-legend-item">'
						content +=
							'<i class="chart-legend-indicator" style="background-color: ' +
							bgColor +
							'"></i>'
						content += label
						content += "</span>"
					})

					return content
				},
			},
		},
	}

	// yAxes
	Chart.scaleService.updateScaleDefaults("linear", {
		gridLines: {
			borderDash: [2],
			borderDashOffset: [2],
			color: mode === "dark" ? colors.gray[900] : colors.gray[300],
			drawBorder: false,
			drawTicks: false,
			lineWidth: 0,
			zeroLineWidth: 0,
			zeroLineColor:
				mode === "dark" ? colors.gray[900] : colors.gray[300],
			zeroLineBorderDash: [2],
			zeroLineBorderDashOffset: [2],
		},
		ticks: {
			beginAtZero: true,
			padding: 10,
			callback: function (value) {
				if (!(value % 10)) {
					return value
				}
			},
		},
	})

	// xAxes
	Chart.scaleService.updateScaleDefaults("category", {
		gridLines: {
			drawBorder: false,
			drawOnChartArea: false,
			drawTicks: false,
		},
		ticks: {
			padding: 20,
		},
	})

	return options
}

// Parse global options
export function parseOptions(parent, options) {
	for (const item in options) {
		if (typeof options[item] !== "object") {
			parent[item] = options[item]
		} else {
			parseOptions(parent[item], options[item])
		}
	}
}

// Example 1 of Chart inside src/views/Index.js (Sales value - Card)
export const chartExample1 = {
	options: {
		scales: {
			yAxes: [
				{
					gridLines: {
						color: colors.gray[900],
						zeroLineColor: colors.gray[900],
					},
					ticks: {
						callback: function (value) {
							if (!(value % 10)) {
								return "$" + value + "k"
							}
						},
					},
				},
			],
		},
		tooltips: {
			callbacks: {
				label: function (item, data) {
					const label = data.datasets[item.datasetIndex].label || ""
					const yLabel = item.yLabel
					let content = ""

					if (data.datasets.length > 1) {
						content += label
					}

					content += "$" + yLabel + "k"
					return content
				},
			},
		},
	},
	data1: (canvas) => {
		return {
			labels: ["May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
			datasets: [
				{
					label: "Performance",
					data: [0, 20, 10, 30, 15, 40, 20, 60, 60],
				},
			],
		}
	},
	data2: (canvas) => {
		return {
			labels: ["May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
			datasets: [
				{
					label: "Performance",
					data: [0, 20, 5, 25, 10, 30, 15, 40, 40],
				},
			],
		}
	},
}

// Example 2 of Chart inside src/views/Index.js (Total orders - Card)
export const chartExample2 = {
	options: {
		scales: {
			yAxes: [
				{
					ticks: {
						callback: function (value) {
							if (!(value % 10)) {
								//return '$' + value + 'k'
								return value
							}
						},
					},
				},
			],
		},
		tooltips: {
			callbacks: {
				label: function (item, data) {
					const label = data.datasets[item.datasetIndex].label || ""
					const yLabel = item.yLabel
					let content = ""
					if (data.datasets.length > 1) {
						content += label
					}
					content += yLabel
					return content
				},
			},
		},
	},
	data: {
		labels: ["Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
		datasets: [
			{
				label: "Sales",
				data: [25, 20, 30, 22, 17, 29],
				maxBarThickness: 10,
			},
		],
	},
}
