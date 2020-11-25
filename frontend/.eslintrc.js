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
		"prettier/prettier": ["warn"],
		"no-console": "off",
		"@typescript-eslint/explicit-function-return-type": "off",
		"react/jsx-filename-extension": "off",
		"react/jsx-indent": "off",
		"import/extensions": "off",
		"@typescript-eslint/camelcase": "off",
	},
}
