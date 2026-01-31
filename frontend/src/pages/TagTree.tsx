import { useEffect } from 'react'
import { observer } from 'mobx-react-lite'
import { tagTreeStore, type TreeNode, type TimeRangeType } from '../stores/tagTreeStore'
import { formatDuration } from '../lib/formatDuration'

const rangeOptions: { value: TimeRangeType; label: string }[] = [
  { value: 'day', label: 'Day' },
  { value: 'week', label: 'Week' },
  { value: 'month', label: 'Month' },
]

const TagTreeTimeRangeSelector = observer(function TagTreeTimeRangeSelector() {
  return (
    <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-6">
      <div className="flex items-center gap-2">
        {rangeOptions.map((option) => (
          <button
            key={option.value}
            onClick={() => tagTreeStore.setRangeType(option.value)}
            className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
              tagTreeStore.rangeType === option.value
                ? 'bg-indigo-600 text-white'
                : 'bg-white text-gray-700 hover:bg-gray-100 border border-gray-300'
            }`}
          >
            {option.label}
          </button>
        ))}
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={() => tagTreeStore.navigatePrevious()}
          className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
          aria-label="Previous"
        >
          <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>

        <button
          onClick={() => tagTreeStore.navigateToday()}
          className="px-4 py-2 text-sm font-medium bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-100 transition-colors"
        >
          Today
        </button>

        <button
          onClick={() => tagTreeStore.navigateNext()}
          className="p-2 rounded-md bg-white border border-gray-300 hover:bg-gray-100 transition-colors"
          aria-label="Next"
        >
          <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>

        <span className="ml-4 text-lg font-medium text-gray-900 min-w-48 text-center">
          {tagTreeStore.rangeLabel}
        </span>
      </div>
    </div>
  )
})

interface TreeNodeRowProps {
  node: TreeNode
  depth: number
  maxDuration: number
}

const TreeNodeRow = observer(function TreeNodeRow({ node, depth, maxDuration }: TreeNodeRowProps) {
  const hasChildren = node.children.size > 0
  const isExpanded = tagTreeStore.expandedPaths.has(node.fullPath)
  const percentage = maxDuration > 0 ? (node.duration / maxDuration) * 100 : 0

  return (
    <>
      <div
        className={`flex items-center py-2 px-3 hover:bg-gray-50 border-b border-gray-100 ${
          depth === 0 ? 'bg-gray-50' : ''
        }`}
        style={{ paddingLeft: `${depth * 20 + 12}px` }}
      >
        {/* Expand/Collapse button */}
        <button
          onClick={() => hasChildren && tagTreeStore.toggleExpanded(node.fullPath)}
          className={`w-6 h-6 flex items-center justify-center mr-2 rounded transition-colors ${
            hasChildren ? 'hover:bg-gray-200 cursor-pointer' : 'cursor-default'
          }`}
          disabled={!hasChildren}
        >
          {hasChildren ? (
            <svg
              className={`w-4 h-4 text-gray-500 transition-transform ${isExpanded ? 'rotate-90' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          ) : (
            <span className="w-4 h-4" />
          )}
        </button>

        {/* Node name */}
        <span className={`flex-shrink-0 min-w-32 ${depth === 0 ? 'font-semibold text-gray-900' : 'text-gray-700'}`}>
          {node.name}
        </span>

        {/* Progress bar */}
        <div className="flex-1 mx-4">
          <div className="h-4 bg-gray-200 rounded-full overflow-hidden">
            <div
              className="h-full bg-indigo-500 rounded-full transition-all duration-300"
              style={{ width: `${percentage}%` }}
            />
          </div>
        </div>

        {/* Duration */}
        <span className="flex-shrink-0 text-sm font-medium text-gray-600 min-w-20 text-right">
          {formatDuration(node.duration)}
        </span>

        {/* Percentage */}
        <span className="flex-shrink-0 text-sm text-gray-400 min-w-16 text-right">
          {percentage.toFixed(1)}%
        </span>
      </div>

      {/* Children */}
      {isExpanded &&
        Array.from(node.children.values())
          .sort((a, b) => b.duration - a.duration)
          .map((child) => (
            <TreeNodeRow
              key={child.fullPath}
              node={child}
              depth={depth + 1}
              maxDuration={maxDuration}
            />
          ))}
    </>
  )
})

export const TagTree = observer(function TagTree() {
  useEffect(() => {
    void tagTreeStore.fetchData()
  }, [])

  const { tagTree, maxTagDuration, isLoading, error } = tagTreeStore

  const sortedTags = Array.from(tagTree.values()).sort((a, b) => b.duration - a.duration)

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900">Tag Tree</h1>
        <p className="mt-2 text-gray-600">
          Explore your tags in a hierarchical tree view
        </p>
      </div>

      <TagTreeTimeRangeSelector />

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-800">{error}</p>
        </div>
      )}

      {/* Toolbar */}
      <div className="mb-4 flex gap-2">
        <button
          onClick={() => tagTreeStore.expandAll()}
          className="px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
        >
          Expand All
        </button>
        <button
          onClick={() => tagTreeStore.collapseAll()}
          className="px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
        >
          Collapse All
        </button>
      </div>

      {/* Tree */}
      <div className="bg-white rounded-lg shadow-lg overflow-hidden">
        {isLoading ? (
          <div className="h-96 flex items-center justify-center">
            <div className="flex items-center gap-3 text-gray-500">
              <svg className="animate-spin h-6 w-6" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              <span>Loading...</span>
            </div>
          </div>
        ) : sortedTags.length === 0 ? (
          <div className="h-96 flex items-center justify-center text-gray-400">
            <p>No tag data available for this time range</p>
          </div>
        ) : (
          <div className="divide-y divide-gray-100">
            {/* Header */}
            <div className="flex items-center py-3 px-3 bg-gray-100 text-sm font-semibold text-gray-600">
              <span className="w-6 mr-2" />
              <span className="flex-shrink-0 min-w-32">Tag</span>
              <span className="flex-1 mx-4 text-center">Duration</span>
              <span className="flex-shrink-0 min-w-20 text-right">Time</span>
              <span className="flex-shrink-0 min-w-16 text-right">%</span>
            </div>

            {/* Tree nodes */}
            {sortedTags.map((node) => (
              <TreeNodeRow
                key={node.fullPath}
                node={node}
                depth={0}
                maxDuration={maxTagDuration}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
})
