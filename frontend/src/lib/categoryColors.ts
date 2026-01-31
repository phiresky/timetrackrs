/**
 * Color palette for categories
 */
export const CATEGORY_COLORS: Record<string, string> = {
  'Productivity': '#22c55e',
  'Communication': '#3b82f6',
  'Entertainment': '#f59e0b',
  'Development': '#8b5cf6',
  'Browsing': '#06b6d4',
  'Social': '#ec4899',
  'Other': '#6b7280',
}

const FALLBACK_COLORS = [
  '#ef4444',
  '#f97316',
  '#eab308',
  '#84cc16',
  '#14b8a6',
  '#0ea5e9',
  '#6366f1',
  '#a855f7',
  '#d946ef',
  '#f43f5e',
]

/**
 * Get a color for a category, using predefined colors if available
 * or falling back to a color from the palette based on index.
 */
export function getColorForCategory(category: string, index: number): string {
  if (CATEGORY_COLORS[category]) {
    return CATEGORY_COLORS[category]
  }
  return FALLBACK_COLORS[index % FALLBACK_COLORS.length]
}
