import React, { useContext, useEffect } from "react"
import { Link, StaticRouter } from "react-router-dom"
import { Routes } from "./Routes"
import Modal from "react-modal"
import { observer, useLocalStore } from "mobx-react"
import { reaction } from "mobx"

type ModalContextType = { currentLink: string | null }

const ModalContext = React.createContext<ModalContextType>({
	currentLink: null,
})

export const ModalLink: React.FC<{ to: string }> = ({ to, children }) => {
	const context = useContext(ModalContext)
	return (
		<Link
			to={to}
			onClick={(e) => {
				e.preventDefault()
				context.currentLink = to
			}}
		>
			{children}
		</Link>
	)
}

export const MaybeModal: React.FC<{ appElement: HTMLElement }> = observer(
	({ appElement, children }) => {
		const store = useLocalStore<ModalContextType>(() => {
			const x = new URLSearchParams(location.hash.substr(1))
			const currentLink = x.get("modal") || null
			return { currentLink }
		})
		useEffect(() =>
			reaction(
				() => store.currentLink,
				(link) => {
					const x = new URLSearchParams(location.hash.substr(1))
					if (link) x.set("modal", link)
					else x.delete("modal")
					location.hash = x.toString()
				},
			),
		)

		return (
			<ModalContext.Provider value={store}>
				{children}
				{store.currentLink && (
					<Modal
						isOpen={true}
						appElement={appElement}
						onRequestClose={(e) => (store.currentLink = null)}
					>
						<StaticRouter location={store.currentLink}>
							<Routes />
						</StaticRouter>
					</Modal>
				)}
			</ModalContext.Provider>
		)
	},
)
