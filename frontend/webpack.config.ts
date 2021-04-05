// @ts-nocheck
import MiniCssExtractPlugin from "mini-css-extract-plugin"
import * as path from "path"
import * as webpack from "webpack"

const production = process.env.NODE_ENV === "production"
const config: webpack.Configuration = {
	entry: "./src/main.tsx",
	mode: production ? "production" : "development",
	devtool: "source-map", //production ? "source-map" : "eval-source-map",
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "main.js",
		publicPath: "",
	},
	resolve: {
		extensions: [".tsx", ".ts", ".js"],
	},
	plugins: [new MiniCssExtractPlugin()],
	module: {
		rules: [
			{
				test: /\.css$/i,
				use: [
					MiniCssExtractPlugin.loader,
					{ loader: "css-loader", options: { sourceMap: true } },
				],
			},
			{
				test: /\.s[ac]ss$/i,
				use: [
					MiniCssExtractPlugin.loader,
					// Translates CSS into CommonJS
					{ loader: "css-loader", options: { sourceMap: true } },
					// Compiles Sass to CSS
					"sass-loader",
				],
			},
			{
				test: /\.tsx?$/,
				loader: "babel-loader",
			},
			{
				test: /\.(png|svg|jpg|jpeg|gif|woff|woff2|ttf|svg|eot)$/i,
				type: "asset/resource",
			},
		],
	},
	devServer: {
		publicPath: "/dist",
		historyApiFallback: { index: "index.html", disableDotRule: true },
	},
}

export default config
