/**
 * Formats a duration in milliseconds to a human-readable string.
 * Examples: "4h 32m", "45m", "1h", "2h 5m"
 */
export function formatDuration(ms: number): string {
  if (ms <= 0) return '0m'

  const totalMinutes = Math.round(ms / (1000 * 60))
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60

  if (hours === 0) {
    return `${minutes}m`
  }

  if (minutes === 0) {
    return `${hours}h`
  }

  return `${hours}h ${minutes}m`
}

/**
 * Formats a percentage value to a string with one decimal place.
 */
export function formatPercentage(value: number): string {
  return `${value.toFixed(1)}%`
}
