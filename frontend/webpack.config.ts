import MiniCssExtractPlugin from "mini-css-extract-plugin"
import * as path from "path"
import * as webpack from "webpack"

const config: webpack.Configuration = {
	entry: "./src/main.tsx",
	mode: process.env.NODE_ENV === "production" ? "production" : "development",
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "main.js",
	},
	resolve: {
		extensions: [".tsx", ".ts", ".js"],
	},
	plugins: [new MiniCssExtractPlugin() as any],
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [
					MiniCssExtractPlugin.loader,
					// Translates CSS into CommonJS
					"css-loader",
					// Compiles Sass to CSS
					"sass-loader",
				],
			},
			{
				test: /\.tsx?$/,
				loader: "babel-loader",
			},
		],
	},
}

export default config
