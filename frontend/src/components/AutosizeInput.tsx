import React, { ChangeEventHandler, Component, CSSProperties } from "react"

const sizerStyle: CSSProperties = {
	position: "absolute",
	top: 0,
	left: 0,
	visibility: "hidden",
	height: 0,
	overflow: "scroll",
	whiteSpace: "pre",
}

const copyStyles = (styles: CSSStyleDeclaration, node: HTMLElement) => {
	node.style.fontSize = styles.fontSize
	node.style.fontFamily = styles.fontFamily
	node.style.fontWeight = styles.fontWeight
	node.style.fontStyle = styles.fontStyle
	node.style.letterSpacing = styles.letterSpacing
	node.style.textTransform = styles.textTransform
}

const isIE =
	typeof window !== "undefined" && window.navigator
		? /MSIE |Trident\/|Edge\//.test(window.navigator.userAgent)
		: false

declare const ResizeObserver: typeof MutationObserver
class AutosizeInput extends Component<Props, { inputWidth: string | number }> {
	input = React.createRef<HTMLInputElement>()
	mounted = false
	placeHolderSizer = React.createRef<HTMLDivElement>()
	sizer = React.createRef<HTMLDivElement>()

	constructor(props: Props) {
		super(props)
		this.state = {
			inputWidth: props.minWidth,
		}
	}
	componentDidMount(): void {
		this.mounted = true
		this.copyInputStyles()
		this.updateInputWidth()

		if (this.input.current)
			new ResizeObserver(() => this.updateInputWidth()).observe(
				this.input.current,
			)
	}
	componentDidUpdate(
		prevProps: AutosizeInput["props"],
		prevState: AutosizeInput["state"],
	): void {
		if (prevState.inputWidth !== this.state.inputWidth) {
			if (typeof this.props.onAutosize === "function") {
				this.props.onAutosize(this.state.inputWidth)
			}
		}
		this.updateInputWidth()
	}
	componentWillUnmount(): void {
		this.mounted = false
	}
	copyInputStyles(): void {
		if (!this.mounted || !window.getComputedStyle) {
			return
		}
		const inputStyles =
			this.input.current && window.getComputedStyle(this.input.current)
		if (!inputStyles) {
			return
		}
		console.log("copying input styles")
		if (this.sizer.current) copyStyles(inputStyles, this.sizer.current)
		if (this.placeHolderSizer.current) {
			copyStyles(inputStyles, this.placeHolderSizer.current)
		}
	}
	updateInputWidth(): void {
		if (
			!this.mounted ||
			!this.sizer.current ||
			typeof this.sizer.current.scrollWidth === "undefined"
		) {
			return
		}
		let newInputWidth
		if (
			this.props.placeholder &&
			(!this.props.value ||
				(this.props.value && this.props.placeholderIsMinWidth))
		) {
			newInputWidth =
				Math.max(
					this.sizer.current.scrollWidth,
					this.placeHolderSizer.current?.scrollWidth || 0,
				) + 2
		} else {
			newInputWidth = (this.sizer.current?.scrollWidth || 0) + 2
		}
		// add extraWidth to the detected width. for number types, this defaults to 16 to allow for the stepper UI
		const extraWidth =
			this.props.type === "number" && this.props.extraWidth === undefined
				? 16
				: parseInt(String(this.props.extraWidth)) || 0
		newInputWidth += extraWidth
		if (newInputWidth < this.props.minWidth) {
			newInputWidth = this.props.minWidth
		}
		if (newInputWidth !== this.state.inputWidth) {
			this.setState({
				inputWidth: newInputWidth,
			})
		}
	}
	getInput() {
		return this.input.current
	}
	focus() {
		this.input.current?.focus()
	}
	blur() {
		this.input.current?.blur()
	}
	select() {
		this.input.current?.select()
	}
	render() {
		const sizerValue = [
			this.props.defaultValue,
			this.props.value,
			"",
		].reduce((previousValue, currentValue) => {
			if (previousValue !== null && previousValue !== undefined) {
				return previousValue
			}
			return currentValue
		})

		const wrapperStyle = { ...(this.props.style || {}) }
		if (!wrapperStyle.display) wrapperStyle.display = "inline-block"

		const {
			extraWidth,
			inputClassName,

			inputStyle,
			minWidth,
			onAutosize,
			placeholderIsMinWidth,
			..._inputProps
		} = this.props

		const inputProps = {
			..._inputProps,
			className: this.props.inputClassName,
			style: {
				boxSizing: "content-box",
				width: `${this.state.inputWidth}px`,
				...inputStyle,
			} as CSSProperties,
		}
		console.log(inputProps)

		return (
			<div className={this.props.className} style={wrapperStyle}>
				<input {...inputProps} ref={this.input} />
				<div ref={this.sizer} style={sizerStyle}>
					{sizerValue}
				</div>
				{this.props.placeholder ? (
					<div ref={this.placeHolderSizer} style={sizerStyle}>
						{this.props.placeholder}
					</div>
				) : null}
			</div>
		)
	}
}

type Props = {
	className?: string // className for the outer element
	defaultValue?: string | number // default field value
	extraWidth?: number | string
	id?: string // id to use for the input, can be set for consistent snapshots
	inputClassName?: string // className for the input element
	inputStyle?: CSSProperties // css styles for the input element
	minWidth: number | string
	onAutosize?: (newWidth: number | string) => void // onAutosize handler: function(newWidth) {}
	onChange?: ChangeEventHandler // onChange handler: function(event) {}
	placeholder?: string // placeholder text
	placeholderIsMinWidth?: boolean // don't collapse size to less than the placeholder
	style?: CSSProperties // css styles for the outer element
	value?: string | number // field value
} & React.InputHTMLAttributes<HTMLInputElement>

export default AutosizeInput
