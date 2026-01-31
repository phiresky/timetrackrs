import { makeAutoObservable, runInAction } from 'mobx'
import { getTagRules, saveTagRules, getKnownTags } from '../lib/api'
import type { TagRuleGroup, TagRuleGroupV1, TagRuleWithMeta, TagRule } from '../server'

export interface EditableTagRuleGroup {
  global_id: string
  data: TagRuleGroupV1
  isExpanded: boolean
}

class RuleEditorStore {
  groups: EditableTagRuleGroup[] = []
  originalGroups: TagRuleGroup[] = []
  knownTags: string[] = []
  isLoading = false
  isSaving = false
  error: string | null = null
  editingRuleIndex: { groupId: string; ruleIndex: number } | null = null

  constructor() {
    makeAutoObservable(this)
  }

  get hasUnsavedChanges(): boolean {
    if (this.groups.length !== this.originalGroups.length) return true
    
    for (let i = 0; i < this.groups.length; i++) {
      const current = this.groups[i]
      const original = this.originalGroups[i]
      
      if (current.global_id !== original.global_id) return true
      
      const currentData = current.data
      const originalData = original.data.data
      
      if (currentData.enabled !== originalData.enabled) return true
      if (currentData.name !== originalData.name) return true
      if (currentData.description !== originalData.description) return true
      if (currentData.rules.length !== originalData.rules.length) return true
      
      if (JSON.stringify(currentData.rules) !== JSON.stringify(originalData.rules)) {
        return true
      }
    }
    
    return false
  }

  async fetchData() {
    this.isLoading = true
    this.error = null

    try {
      const [rulesData, tagsData] = await Promise.all([
        getTagRules(),
        getKnownTags(),
      ])

      runInAction(() => {
        this.originalGroups = JSON.parse(JSON.stringify(rulesData)) as TagRuleGroup[]
        this.groups = rulesData.map((group) => ({
          global_id: group.global_id,
          data: group.data.data,
          isExpanded: false,
        }))
        this.knownTags = tagsData
        this.isLoading = false
      })
    } catch (err) {
      runInAction(() => {
        this.error = err instanceof Error ? err.message : 'Failed to fetch rules'
        this.isLoading = false
      })
    }
  }

  toggleGroupExpanded(groupId: string) {
    const group = this.groups.find((g) => g.global_id === groupId)
    if (group) {
      group.isExpanded = !group.isExpanded
    }
  }

  toggleGroupEnabled(groupId: string) {
    const group = this.groups.find((g) => g.global_id === groupId)
    if (group) {
      group.data.enabled = !group.data.enabled
    }
  }

  toggleRuleEnabled(groupId: string, ruleIndex: number) {
    const group = this.groups.find((g) => g.global_id === groupId)
    if (group && group.data.rules[ruleIndex]) {
      group.data.rules[ruleIndex].enabled = !group.data.rules[ruleIndex].enabled
    }
  }

  setEditingRule(groupId: string, ruleIndex: number) {
    this.editingRuleIndex = { groupId, ruleIndex }
  }

  clearEditingRule() {
    this.editingRuleIndex = null
  }

  updateRule(groupId: string, ruleIndex: number, rule: TagRuleWithMeta) {
    const group = this.groups.find((g) => g.global_id === groupId)
    if (group && group.data.rules[ruleIndex]) {
      group.data.rules[ruleIndex] = rule
    }
  }

  addRule(groupId: string) {
    const group = this.groups.find((g) => g.global_id === groupId)
    if (group && group.data.editable) {
      const newRule: TagRuleWithMeta = {
        enabled: true,
        rule: {
          type: 'HasTag',
          tag: '',
          new_tags: [],
        },
      }
      group.data.rules.push(newRule)
    }
  }

  deleteRule(groupId: string, ruleIndex: number) {
    const group = this.groups.find((g) => g.global_id === groupId)
    if (group && group.data.editable) {
      group.data.rules.splice(ruleIndex, 1)
    }
  }

  discardChanges() {
    this.groups = this.originalGroups.map((group) => ({
      global_id: group.global_id,
      data: JSON.parse(JSON.stringify(group.data.data)) as TagRuleGroupV1,
      isExpanded: this.groups.find((g) => g.global_id === group.global_id)?.isExpanded ?? false,
    }))
    this.editingRuleIndex = null
  }

  async saveChanges() {
    this.isSaving = true
    this.error = null

    try {
      const groupsToSave: TagRuleGroup[] = this.groups.map((group) => ({
        global_id: group.global_id,
        data: {
          version: 'V1' as const,
          data: group.data,
        },
      }))

      await saveTagRules(groupsToSave)

      runInAction(() => {
        this.originalGroups = JSON.parse(JSON.stringify(groupsToSave)) as TagRuleGroup[]
        this.isSaving = false
        this.editingRuleIndex = null
      })
    } catch (err) {
      runInAction(() => {
        this.error = err instanceof Error ? err.message : 'Failed to save rules'
        this.isSaving = false
      })
    }
  }
}

export const ruleEditorStore = new RuleEditorStore()

// Helper functions for working with TagRule types
export function getRuleTypeLabel(rule: TagRule): string {
  switch (rule.type) {
    case 'HasTag':
      return 'Has Tag'
    case 'ExactTagValue':
      return 'Exact Tag Value'
    case 'TagValuePrefix':
      return 'Tag Value Prefix'
    case 'TagRegex':
      return 'Tag Regex'
    case 'InternalFetcher':
      return 'Internal Fetcher'
    case 'ExternalFetcher':
      return 'External Fetcher'
  }
}

export function getRuleSummary(rule: TagRule): string {
  switch (rule.type) {
    case 'HasTag':
      return `tag: ${rule.tag}`
    case 'ExactTagValue':
      return `${rule.tag} = "${rule.value}"`
    case 'TagValuePrefix':
      return `${rule.tag} starts with "${rule.prefix}"`
    case 'TagRegex':
      return `${rule.regexes.length} regex pattern(s)`
    case 'InternalFetcher':
      return `fetcher: ${rule.fetcher_id}`
    case 'ExternalFetcher':
      return `external: ${rule.fetcher_id}`
  }
}

export function getNewTagsSummary(rule: TagRule): string {
  if ('new_tags' in rule && rule.new_tags.length > 0) {
    return rule.new_tags.map((t) => `${t.tag}=${t.value}`).join(', ')
  }
  return ''
}

export const RULE_TYPES: TagRule['type'][] = [
  'HasTag',
  'ExactTagValue',
  'TagValuePrefix',
  'TagRegex',
  'InternalFetcher',
  'ExternalFetcher',
]

export function createEmptyRule(type: TagRule['type']): TagRule {
  switch (type) {
    case 'HasTag':
      return { type: 'HasTag', tag: '', new_tags: [] }
    case 'ExactTagValue':
      return { type: 'ExactTagValue', tag: '', value: '', new_tags: [] }
    case 'TagValuePrefix':
      return { type: 'TagValuePrefix', tag: '', prefix: '', new_tags: [] }
    case 'TagRegex':
      return { type: 'TagRegex', regexes: [], new_tags: [] }
    case 'InternalFetcher':
      return { type: 'InternalFetcher', fetcher_id: '' }
    case 'ExternalFetcher':
      return { type: 'ExternalFetcher', fetcher_id: '' }
  }
}
