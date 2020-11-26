import { Activity } from "./api"

export function totalDuration(entries: Activity[]): number {
	return entries.reduce((sum, b) => sum + b.duration, 0)
}

export function durationToString(duration: number): string {
	if (duration < 60) {
		return `${Math.round(duration)} s`
	}
	duration = Math.round(duration / 60)
	if (duration >= 60)
		return `${Math.round(duration / 60)} h ${duration % 60} min`
	return `${duration} min`
}
