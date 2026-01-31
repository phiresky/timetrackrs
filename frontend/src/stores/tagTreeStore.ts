import { makeAutoObservable, runInAction } from 'mobx'
import { Temporal } from '@js-temporal/polyfill'
import { getTimeRange } from '../lib/api'
import type { SingleExtractedChunk } from '../server'

export type TimeRangeType = 'day' | 'week' | 'month'

export interface TreeNode {
  name: string
  fullPath: string
  duration: number
  children: Map<string, TreeNode>
  expanded: boolean
}

class TagTreeStore {
  rangeType: TimeRangeType = 'week'
  currentDate: Temporal.PlainDate = Temporal.Now.plainDateISO()
  chunks: SingleExtractedChunk[] = []
  isLoading = false
  error: string | null = null
  expandedPaths: Set<string> = new Set()

  constructor() {
    makeAutoObservable(this)
  }

  get rangeStart(): Temporal.PlainDate {
    switch (this.rangeType) {
      case 'day':
        return this.currentDate
      case 'week':
        return this.currentDate.subtract({ days: this.currentDate.dayOfWeek - 1 })
      case 'month':
        return this.currentDate.with({ day: 1 })
    }
  }

  get rangeEnd(): Temporal.PlainDate {
    switch (this.rangeType) {
      case 'day':
        return this.currentDate.add({ days: 1 })
      case 'week':
        return this.rangeStart.add({ weeks: 1 })
      case 'month':
        return this.rangeStart.add({ months: 1 })
    }
  }

  get rangeLabel(): string {
    const start = this.rangeStart
    const end = this.rangeEnd.subtract({ days: 1 })
    
    switch (this.rangeType) {
      case 'day':
        return start.toLocaleString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' })
      case 'week':
        return `${start.toLocaleString('en-US', { month: 'short', day: 'numeric' })} - ${end.toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}`
      case 'month':
        return start.toLocaleString('en-US', { month: 'long', year: 'numeric' })
    }
  }

  get tagTree(): Map<string, TreeNode> {
    const rootNodes = new Map<string, TreeNode>()
    
    // Aggregate durations by tag name and value
    const tagData = new Map<string, Map<string, number>>()
    
    for (const chunk of this.chunks) {
      for (const [tagName, tagValue, duration] of chunk.tags) {
        if (!tagData.has(tagName)) {
          tagData.set(tagName, new Map())
        }
        const tagValues = tagData.get(tagName)!
        const current = tagValues.get(tagValue) || 0
        tagValues.set(tagValue, current + duration)
      }
    }

    // Build tree for each tag type
    for (const [tagName, values] of tagData) {
      const tagNode: TreeNode = {
        name: tagName,
        fullPath: tagName,
        duration: 0,
        children: new Map(),
        expanded: this.expandedPaths.has(tagName),
      }

      for (const [value, duration] of values) {
        // Split value by "/" to create hierarchy
        const parts = value.split('/')
        let currentNode = tagNode
        let currentPath = tagName

        for (let i = 0; i < parts.length; i++) {
          const part = parts[i]
          currentPath = `${currentPath}/${part}`
          
          if (!currentNode.children.has(part)) {
            currentNode.children.set(part, {
              name: part,
              fullPath: currentPath,
              duration: 0,
              children: new Map(),
              expanded: this.expandedPaths.has(currentPath),
            })
          }
          
          currentNode = currentNode.children.get(part)!
          
          // Add duration to this level and all parents will be summed later
          if (i === parts.length - 1) {
            currentNode.duration += duration
          }
        }
        
        tagNode.duration += duration
      }

      // Propagate durations up the tree
      this.propagateDurations(tagNode)
      rootNodes.set(tagName, tagNode)
    }

    return rootNodes
  }

  private propagateDurations(node: TreeNode): number {
    if (node.children.size === 0) {
      return node.duration
    }

    let childSum = 0
    for (const child of node.children.values()) {
      childSum += this.propagateDurations(child)
    }

    // If node has its own duration (leaf was here), keep it
    // Otherwise, it's the sum of children
    if (node.duration === 0) {
      node.duration = childSum
    }

    return node.duration
  }

  get maxTagDuration(): number {
    let max = 0
    for (const node of this.tagTree.values()) {
      if (node.duration > max) {
        max = node.duration
      }
    }
    return max
  }

  toggleExpanded(path: string) {
    if (this.expandedPaths.has(path)) {
      this.expandedPaths.delete(path)
    } else {
      this.expandedPaths.add(path)
    }
  }

  expandAll() {
    const addPaths = (node: TreeNode) => {
      this.expandedPaths.add(node.fullPath)
      for (const child of node.children.values()) {
        addPaths(child)
      }
    }
    for (const node of this.tagTree.values()) {
      addPaths(node)
    }
  }

  collapseAll() {
    this.expandedPaths.clear()
  }

  setRangeType(type: TimeRangeType) {
    this.rangeType = type
    void this.fetchData()
  }

  navigatePrevious() {
    switch (this.rangeType) {
      case 'day':
        this.currentDate = this.currentDate.subtract({ days: 1 })
        break
      case 'week':
        this.currentDate = this.currentDate.subtract({ weeks: 1 })
        break
      case 'month':
        this.currentDate = this.currentDate.subtract({ months: 1 })
        break
    }
    void this.fetchData()
  }

  navigateNext() {
    switch (this.rangeType) {
      case 'day':
        this.currentDate = this.currentDate.add({ days: 1 })
        break
      case 'week':
        this.currentDate = this.currentDate.add({ weeks: 1 })
        break
      case 'month':
        this.currentDate = this.currentDate.add({ months: 1 })
        break
    }
    void this.fetchData()
  }

  navigateToday() {
    this.currentDate = Temporal.Now.plainDateISO()
    void this.fetchData()
  }

  async fetchData() {
    this.isLoading = true
    this.error = null

    try {
      const after = this.rangeStart
        .toZonedDateTime({ timeZone: Temporal.Now.timeZoneId(), plainTime: new Temporal.PlainTime(0, 0, 0) })
        .toInstant()
      const before = this.rangeEnd
        .toZonedDateTime({ timeZone: Temporal.Now.timeZoneId(), plainTime: new Temporal.PlainTime(0, 0, 0) })
        .toInstant()

      const data = await getTimeRange({
        after,
        before,
        tag: null,
      })

      runInAction(() => {
        this.chunks = data
        this.isLoading = false
      })
    } catch (err) {
      runInAction(() => {
        this.error = err instanceof Error ? err.message : 'Failed to fetch data'
        this.isLoading = false
      })
    }
  }
}

export const tagTreeStore = new TagTreeStore()
