import React from "react"
import { render } from "react-dom"
import { BrowserRouter } from "react-router-dom"
import { MaybeModal } from "./components/ModalLink"
import { Routes } from "./components/Routes"
import "./style.scss"

const appElement = document.getElementById("root")

function Main() {
	if (!appElement) throw Error("could not find app container")
	return (
		<BrowserRouter>
			<MaybeModal appElement={appElement}>
				<Routes />
			</MaybeModal>
		</BrowserRouter>
	)
}
render(<Main />, appElement)
