// @ts-nocheck
import MiniCssExtractPlugin from "mini-css-extract-plugin"
import * as path from "path"
import * as webpack from "webpack"

const production = process.env.NODE_ENV === "production"
const config: webpack.Configuration = {
	entry: "./src/main.tsx",
	mode: production ? "production" : "development",
	devtool: production ? "source-map" : "eval-source-map",
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
				test: /\.css$/i,
				use: [MiniCssExtractPlugin.loader, "css-loader"],
			},
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
	devServer: {
		publicPath: "/dist",
		historyApiFallback: { index: "index.html", disableDotRule: true },
	},
}

export default config
