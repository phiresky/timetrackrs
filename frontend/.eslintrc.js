const isEditor = process.env.ELECTRON_RUN_AS_NODE && process.env.VSCODE_CWD
module.exports = {
	extends: [
		"eslint:recommended",
		"plugin:react/recommended",
		"plugin:@typescript-eslint/eslint-recommended",
		"plugin:@typescript-eslint/recommended",
		"plugin:@typescript-eslint/recommended-requiring-type-checking",
		"prettier",
		"prettier/@typescript-eslint",
	],
	parserOptions: { tsconfigRootDir: __dirname, project: ["./tsconfig.json"] },
	plugins: ["prettier", "@typescript-eslint"],
	env: { es6: true, browser: true, node: true },
	parser: "@typescript-eslint/parser",
	rules: {
		"prettier/prettier": isEditor ? "off" : ["warn"],
		"no-console": "off",
		"react/jsx-filename-extension": "off",
		"react/jsx-indent": "off",
		"import/extensions": "off",
		"@typescript-eslint/camelcase": "off",
		"react/prop-types": "off",
		"react/display-name": "off",
	},
}
