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
import { Chart, ChartItem, TooltipItem } from "chart.js"

const mode: "dark" | "light" = "light" //(themeMode) ? themeMode : 'light';
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
	return {
		responsive: true,
		maintainAspectRatio: false,
		defaultColor: mode === "dark" ? colors.gray[700] : colors.gray[600],
		defaultFontColor: mode === "dark" ? colors.gray[700] : colors.gray[600],
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
				borderColor: mode === "dark" ? colors.gray[800] : colors.white,
				borderWidth: 4,
			},
		},
		tooltips: {
			enabled: true,
			mode: "index",
			intersect: false,
		},
	}
}

// Parse global options
export function parseOptions(parent: any, options: any) {
	for (const item in options) {
		if (typeof options[item] !== "object") {
			parent[item] = options[item]
		} else {
			parseOptions(parent[item], options[item])
		}
	}
}
