import { Activity } from "./main"

export function totalDuration(entries: Activity[]) {
	return entries.reduce((sum, b) => sum + b.duration, 0)
}

export function durationToString(duration: number) {
	duration = Math.round(duration / 60)
	if (duration >= 60)
		return Math.round(duration / 60) + " h " + (duration % 60) + " min"
	return duration + " min"
}
