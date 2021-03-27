import { useLocalObservable, observer } from "mobx-react"
import * as React from "react"
import { useEffect } from "react"
import { progressEvents } from "../api"
import { ProgressReport } from "../server"

export const ProgressPopup = observer(() => {
	const obs = useLocalObservable(() => ({
		progresses: new Map<string, ProgressReport>(),
		es: null as EventSource | null,
		updateProgress(p: ProgressReport) {
			if (p.state.length === 0) {
				obs.progresses.delete(p.call_id)
				return
			}
			const prog = obs.progresses.get(p.call_id)
			if (prog) Object.assign(prog, p)
			else {
				obs.progresses.set(p.call_id, p)
			}
		},
	}))
	useEffect(() => {
		console.log("opening events connection")
		obs.es = progressEvents((p) => obs.updateProgress(p))
		return () => {
			console.log("closing events connection")
			obs.es?.close()
		}
	}, [])
	if (obs.progresses.size === 0) return null
	return (
		<div className="progress-popup">
			{[...obs.progresses.values()].map((prog) => (
				<div key={prog.call_id}>
					<p>Task {prog.call_desc}</p>
					{prog.state.map((state, i) => (
						<div key={i}>
							{state.desc}:{" "}
							{state.total
								? `${(
										(state.current / state.total) *
										100
								  ).toFixed(0)}% (${state.current}/${
										state.total
								  })`
								: state.current}
						</div>
					))}
				</div>
			))}
		</div>
	)
})
