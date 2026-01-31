import { useEffect, useState } from 'react'
import { observer } from 'mobx-react-lite'
import {
  ruleEditorStore,
  getRuleTypeLabel,
  getRuleSummary,
  getNewTagsSummary,
  RULE_TYPES,
  createEmptyRule,
} from '../stores/ruleEditorStore'
import type { TagRule, TagRuleWithMeta, TagValue, TagValueRegex } from '../server'

// Icons as simple components
function ChevronDownIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
    </svg>
  )
}

function ChevronRightIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
    </svg>
  )
}

function PlusIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
    </svg>
  )
}

function TrashIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
    </svg>
  )
}

function PencilIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
    </svg>
  )
}

// Tag input component with autocomplete
interface TagInputProps {
  value: string
  onChange: (value: string) => void
  knownTags: string[]
  placeholder?: string
  className?: string
  disabled?: boolean
}

function TagInput({ value, onChange, knownTags, placeholder, className, disabled }: TagInputProps) {
  const [showSuggestions, setShowSuggestions] = useState(false)
  const filteredTags = knownTags.filter(
    (tag) => tag.toLowerCase().includes(value.toLowerCase()) && tag !== value
  )

  return (
    <div className="relative">
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onFocus={() => !disabled && setShowSuggestions(true)}
        onBlur={() => setTimeout(() => setShowSuggestions(false), 200)}
        placeholder={placeholder}
        className={className}
        disabled={disabled}
      />
      {showSuggestions && !disabled && filteredTags.length > 0 && (
        <ul className="absolute z-10 mt-1 max-h-40 w-full overflow-auto rounded-md bg-white py-1 text-sm shadow-lg ring-1 ring-black ring-opacity-5">
          {filteredTags.slice(0, 10).map((tag) => (
            <li
              key={tag}
              className="cursor-pointer px-3 py-2 hover:bg-indigo-600 hover:text-white"
              onMouseDown={() => onChange(tag)}
            >
              {tag}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

// New tag editor component
interface NewTagsEditorProps {
  newTags: TagValue[]
  onChange: (newTags: TagValue[]) => void
  knownTags: string[]
  disabled?: boolean
}

function NewTagsEditor({ newTags, onChange, knownTags, disabled }: NewTagsEditorProps) {
  const addNewTag = () => {
    onChange([...newTags, { tag: '', value: '' }])
  }

  const updateTag = (index: number, field: 'tag' | 'value', value: string) => {
    const updated = [...newTags]
    updated[index] = { ...updated[index], [field]: value }
    onChange(updated)
  }

  const removeTag = (index: number) => {
    onChange(newTags.filter((_, i) => i !== index))
  }

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700">New Tags to Add</label>
      {newTags.map((newTag, index) => (
        <div key={index} className="flex gap-2 items-center">
          <TagInput
            value={newTag.tag}
            onChange={(v) => updateTag(index, 'tag', v)}
            knownTags={knownTags}
            placeholder="Tag name"
            className="flex-1 rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
            disabled={disabled}
          />
          <input
            type="text"
            value={newTag.value}
            onChange={(e) => updateTag(index, 'value', e.target.value)}
            placeholder="Value"
            className="flex-1 rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
            disabled={disabled}
          />
          {!disabled && (
            <button
              type="button"
              onClick={() => removeTag(index)}
              className="p-1 text-red-500 hover:text-red-700"
            >
              <TrashIcon className="h-4 w-4" />
            </button>
          )}
        </div>
      ))}
      {!disabled && (
        <button
          type="button"
          onClick={addNewTag}
          className="inline-flex items-center gap-1 text-sm text-indigo-600 hover:text-indigo-800"
        >
          <PlusIcon className="h-4 w-4" />
          Add tag
        </button>
      )}
    </div>
  )
}

// Regex editor component
interface RegexEditorProps {
  regexes: TagValueRegex[]
  onChange: (regexes: TagValueRegex[]) => void
  knownTags: string[]
  disabled?: boolean
}

function RegexEditor({ regexes, onChange, knownTags, disabled }: RegexEditorProps) {
  const addRegex = () => {
    onChange([...regexes, { tag: '', regex: '' }])
  }

  const updateRegex = (index: number, field: 'tag' | 'regex', value: string) => {
    const updated = [...regexes]
    updated[index] = { ...updated[index], [field]: value }
    onChange(updated)
  }

  const removeRegex = (index: number) => {
    onChange(regexes.filter((_, i) => i !== index))
  }

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700">Regex Patterns</label>
      {regexes.map((regex, index) => (
        <div key={index} className="flex gap-2 items-center">
          <TagInput
            value={regex.tag}
            onChange={(v) => updateRegex(index, 'tag', v)}
            knownTags={knownTags}
            placeholder="Tag name"
            className="flex-1 rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
            disabled={disabled}
          />
          <input
            type="text"
            value={regex.regex}
            onChange={(e) => updateRegex(index, 'regex', e.target.value)}
            placeholder="Regex pattern"
            className="flex-1 rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm font-mono disabled:bg-gray-100"
            disabled={disabled}
          />
          {!disabled && (
            <button
              type="button"
              onClick={() => removeRegex(index)}
              className="p-1 text-red-500 hover:text-red-700"
            >
              <TrashIcon className="h-4 w-4" />
            </button>
          )}
        </div>
      ))}
      {!disabled && (
        <button
          type="button"
          onClick={addRegex}
          className="inline-flex items-center gap-1 text-sm text-indigo-600 hover:text-indigo-800"
        >
          <PlusIcon className="h-4 w-4" />
          Add pattern
        </button>
      )}
    </div>
  )
}

// Rule editor modal
interface RuleEditorModalProps {
  rule: TagRuleWithMeta
  onSave: (rule: TagRuleWithMeta) => void
  onCancel: () => void
  knownTags: string[]
  editable: boolean
}

function RuleEditorModal({ rule, onSave, onCancel, knownTags, editable }: RuleEditorModalProps) {
  const [localRule, setLocalRule] = useState<TagRuleWithMeta>(JSON.parse(JSON.stringify(rule)) as TagRuleWithMeta)

  const handleTypeChange = (type: TagRule['type']) => {
    setLocalRule({
      ...localRule,
      rule: createEmptyRule(type),
    })
  }

  const updateRuleField = (field: string, value: unknown) => {
    setLocalRule({
      ...localRule,
      rule: { ...localRule.rule, [field]: value } as TagRule,
    })
  }

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" onClick={onCancel} />
        <div className="relative transform overflow-hidden rounded-lg bg-white px-4 pb-4 pt-5 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-lg sm:p-6">
          <div className="mb-4">
            <h3 className="text-lg font-semibold text-gray-900">
              {editable ? 'Edit Rule' : 'View Rule'}
            </h3>
          </div>

          <div className="space-y-4">
            {/* Rule enabled toggle */}
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium text-gray-700">Rule Enabled</label>
              <button
                type="button"
                disabled={!editable}
                onClick={() => setLocalRule({ ...localRule, enabled: !localRule.enabled })}
                className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2 ${
                  localRule.enabled ? 'bg-indigo-600' : 'bg-gray-200'
                } ${!editable ? 'opacity-50 cursor-not-allowed' : ''}`}
              >
                <span
                  className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                    localRule.enabled ? 'translate-x-5' : 'translate-x-0'
                  }`}
                />
              </button>
            </div>

            {/* Rule type selector */}
            <div>
              <label className="block text-sm font-medium text-gray-700">Rule Type</label>
              <select
                value={localRule.rule.type}
                onChange={(e) => handleTypeChange(e.target.value as TagRule['type'])}
                disabled={!editable}
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
              >
                {RULE_TYPES.map((type) => (
                  <option key={type} value={type}>
                    {getRuleTypeLabel({ type } as TagRule)}
                  </option>
                ))}
              </select>
            </div>

            {/* Type-specific fields */}
            {localRule.rule.type === 'HasTag' && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Tag</label>
                  <TagInput
                    value={localRule.rule.tag}
                    onChange={(v) => updateRuleField('tag', v)}
                    knownTags={knownTags}
                    placeholder="Enter tag name"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                    disabled={!editable}
                  />
                </div>
                <NewTagsEditor
                  newTags={localRule.rule.new_tags}
                  onChange={(v) => updateRuleField('new_tags', v)}
                  knownTags={knownTags}
                  disabled={!editable}
                />
              </>
            )}

            {localRule.rule.type === 'ExactTagValue' && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Tag</label>
                  <TagInput
                    value={localRule.rule.tag}
                    onChange={(v) => updateRuleField('tag', v)}
                    knownTags={knownTags}
                    placeholder="Enter tag name"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                    disabled={!editable}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Value</label>
                  <input
                    type="text"
                    value={localRule.rule.value}
                    onChange={(e) => updateRuleField('value', e.target.value)}
                    disabled={!editable}
                    placeholder="Exact value to match"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                  />
                </div>
                <NewTagsEditor
                  newTags={localRule.rule.new_tags}
                  onChange={(v) => updateRuleField('new_tags', v)}
                  knownTags={knownTags}
                  disabled={!editable}
                />
              </>
            )}

            {localRule.rule.type === 'TagValuePrefix' && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Tag</label>
                  <TagInput
                    value={localRule.rule.tag}
                    onChange={(v) => updateRuleField('tag', v)}
                    knownTags={knownTags}
                    placeholder="Enter tag name"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                    disabled={!editable}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Prefix</label>
                  <input
                    type="text"
                    value={localRule.rule.prefix}
                    onChange={(e) => updateRuleField('prefix', e.target.value)}
                    disabled={!editable}
                    placeholder="Value prefix to match"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                  />
                </div>
                <NewTagsEditor
                  newTags={localRule.rule.new_tags}
                  onChange={(v) => updateRuleField('new_tags', v)}
                  knownTags={knownTags}
                  disabled={!editable}
                />
              </>
            )}

            {localRule.rule.type === 'TagRegex' && (
              <>
                <RegexEditor
                  regexes={localRule.rule.regexes}
                  onChange={(v) => updateRuleField('regexes', v)}
                  knownTags={knownTags}
                  disabled={!editable}
                />
                <NewTagsEditor
                  newTags={localRule.rule.new_tags}
                  onChange={(v) => updateRuleField('new_tags', v)}
                  knownTags={knownTags}
                  disabled={!editable}
                />
              </>
            )}

            {localRule.rule.type === 'InternalFetcher' && (
              <div>
                <label className="block text-sm font-medium text-gray-700">Fetcher ID</label>
                <input
                  type="text"
                  value={localRule.rule.fetcher_id}
                  onChange={(e) => updateRuleField('fetcher_id', e.target.value)}
                  disabled={!editable}
                  placeholder="Internal fetcher ID"
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                />
              </div>
            )}

            {localRule.rule.type === 'ExternalFetcher' && (
              <div>
                <label className="block text-sm font-medium text-gray-700">Fetcher ID</label>
                <input
                  type="text"
                  value={localRule.rule.fetcher_id}
                  onChange={(e) => updateRuleField('fetcher_id', e.target.value)}
                  disabled={!editable}
                  placeholder="External fetcher ID"
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 text-sm disabled:bg-gray-100"
                />
              </div>
            )}
          </div>

          <div className="mt-6 flex justify-end gap-3">
            <button
              type="button"
              onClick={onCancel}
              className="rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
            >
              {editable ? 'Cancel' : 'Close'}
            </button>
            {editable && (
              <button
                type="button"
                onClick={() => onSave(localRule)}
                className="rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
              >
                Save
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

// Individual rule row
interface RuleRowProps {
  rule: TagRuleWithMeta
  index: number
  groupId: string
  editable: boolean
  onEdit: () => void
  onToggle: () => void
  onDelete: () => void
}

const RuleRow = observer(function RuleRow({
  rule,
  index,
  editable,
  onEdit,
  onToggle,
  onDelete,
}: RuleRowProps) {
  const newTagsSummary = getNewTagsSummary(rule.rule)

  return (
    <div
      className={`flex items-center gap-4 px-4 py-3 border-b border-gray-100 last:border-b-0 ${
        !rule.enabled ? 'opacity-50' : ''
      }`}
    >
      <span className="text-xs font-mono text-gray-400 w-6">{index + 1}</span>
      
      <button
        type="button"
        onClick={onToggle}
        disabled={!editable}
        className={`relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2 ${
          rule.enabled ? 'bg-indigo-600' : 'bg-gray-200'
        } ${!editable ? 'cursor-not-allowed' : ''}`}
      >
        <span
          className={`pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
            rule.enabled ? 'translate-x-4' : 'translate-x-0'
          }`}
        />
      </button>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="inline-flex items-center rounded-md bg-indigo-50 px-2 py-1 text-xs font-medium text-indigo-700 ring-1 ring-inset ring-indigo-700/10">
            {getRuleTypeLabel(rule.rule)}
          </span>
          <span className="text-sm text-gray-600 truncate">{getRuleSummary(rule.rule)}</span>
        </div>
        {newTagsSummary && (
          <div className="mt-1 text-xs text-gray-500">
            → {newTagsSummary}
          </div>
        )}
      </div>

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onEdit}
          className="p-1.5 text-gray-400 hover:text-indigo-600 rounded-md hover:bg-gray-100"
          title={editable ? 'Edit rule' : 'View rule'}
        >
          <PencilIcon className="h-4 w-4" />
        </button>
        {editable && (
          <button
            type="button"
            onClick={onDelete}
            className="p-1.5 text-gray-400 hover:text-red-600 rounded-md hover:bg-gray-100"
            title="Delete rule"
          >
            <TrashIcon className="h-4 w-4" />
          </button>
        )}
      </div>
    </div>
  )
})

// Rule group component
interface RuleGroupCardProps {
  group: {
    global_id: string
    data: {
      name: string
      description: string
      editable: boolean
      enabled: boolean
      rules: TagRuleWithMeta[]
    }
    isExpanded: boolean
  }
}

const RuleGroupCard = observer(function RuleGroupCard({ group }: RuleGroupCardProps) {
  const { knownTags, editingRuleIndex } = ruleEditorStore
  const isEditing =
    editingRuleIndex?.groupId === group.global_id

  const handleEditRule = (ruleIndex: number) => {
    ruleEditorStore.setEditingRule(group.global_id, ruleIndex)
  }

  const handleSaveRule = (rule: TagRuleWithMeta) => {
    if (editingRuleIndex) {
      ruleEditorStore.updateRule(
        editingRuleIndex.groupId,
        editingRuleIndex.ruleIndex,
        rule
      )
    }
    ruleEditorStore.clearEditingRule()
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
      {/* Group header */}
      <div
        className="flex items-center gap-4 px-4 py-4 cursor-pointer hover:bg-gray-50"
        onClick={() => ruleEditorStore.toggleGroupExpanded(group.global_id)}
      >
        <div className="text-gray-400">
          {group.isExpanded ? (
            <ChevronDownIcon className="h-5 w-5" />
          ) : (
            <ChevronRightIcon className="h-5 w-5" />
          )}
        </div>

        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation()
            ruleEditorStore.toggleGroupEnabled(group.global_id)
          }}
          className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2 ${
            group.data.enabled ? 'bg-indigo-600' : 'bg-gray-200'
          }`}
        >
          <span
            className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
              group.data.enabled ? 'translate-x-5' : 'translate-x-0'
            }`}
          />
        </button>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-semibold text-gray-900">{group.data.name}</h3>
            {!group.data.editable && (
              <span className="inline-flex items-center rounded-md bg-gray-50 px-2 py-1 text-xs font-medium text-gray-600 ring-1 ring-inset ring-gray-500/10">
                Read-only
              </span>
            )}
          </div>
          <p className="text-sm text-gray-500 truncate">{group.data.description}</p>
        </div>

        <div className="text-sm text-gray-500">
          {group.data.rules.length} rule{group.data.rules.length !== 1 ? 's' : ''}
        </div>
      </div>

      {/* Expanded rules */}
      {group.isExpanded && (
        <div className="border-t border-gray-200 bg-gray-50">
          {group.data.rules.length === 0 ? (
            <div className="px-4 py-6 text-center text-sm text-gray-500">
              No rules in this group
            </div>
          ) : (
            <div className="bg-white">
              {group.data.rules.map((rule, index) => (
                <RuleRow
                  key={index}
                  rule={rule}
                  index={index}
                  groupId={group.global_id}
                  editable={group.data.editable}
                  onEdit={() => handleEditRule(index)}
                  onToggle={() =>
                    ruleEditorStore.toggleRuleEnabled(group.global_id, index)
                  }
                  onDelete={() =>
                    ruleEditorStore.deleteRule(group.global_id, index)
                  }
                />
              ))}
            </div>
          )}

          {group.data.editable && (
            <div className="px-4 py-3 border-t border-gray-100">
              <button
                type="button"
                onClick={() => ruleEditorStore.addRule(group.global_id)}
                className="inline-flex items-center gap-2 text-sm font-medium text-indigo-600 hover:text-indigo-800"
              >
                <PlusIcon className="h-4 w-4" />
                Add rule
              </button>
            </div>
          )}
        </div>
      )}

      {/* Rule editor modal */}
      {isEditing && editingRuleIndex && (
        <RuleEditorModal
          rule={group.data.rules[editingRuleIndex.ruleIndex]}
          onSave={handleSaveRule}
          onCancel={() => ruleEditorStore.clearEditingRule()}
          knownTags={knownTags}
          editable={group.data.editable}
        />
      )}
    </div>
  )
})

export const RuleEditor = observer(function RuleEditor() {
  const { groups, isLoading, isSaving, error, hasUnsavedChanges } = ruleEditorStore

  useEffect(() => {
    void ruleEditorStore.fetchData()
  }, [])

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Rule Editor</h1>
          <p className="mt-2 text-gray-600">
            Create and edit classification rules for your tracked activities
          </p>
        </div>

        {hasUnsavedChanges && (
          <div className="flex items-center gap-4">
            <span className="text-sm text-amber-600 font-medium">● Unsaved changes</span>
            <button
              type="button"
              onClick={() => ruleEditorStore.discardChanges()}
              disabled={isSaving}
              className="rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 disabled:opacity-50"
            >
              Discard
            </button>
            <button
              type="button"
              onClick={() => void ruleEditorStore.saveChanges()}
              disabled={isSaving}
              className="rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:opacity-50"
            >
              {isSaving ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        )}
      </div>

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-800">{error}</p>
        </div>
      )}

      {isLoading ? (
        <div className="space-y-4">
          {[1, 2, 3].map((i) => (
            <div key={i} className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
              <div className="animate-pulse flex items-center gap-4">
                <div className="h-5 w-5 bg-gray-200 rounded" />
                <div className="h-6 w-11 bg-gray-200 rounded-full" />
                <div className="flex-1">
                  <div className="h-4 bg-gray-200 rounded w-1/4 mb-2" />
                  <div className="h-3 bg-gray-200 rounded w-1/2" />
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : groups.length === 0 ? (
        <div className="bg-white rounded-lg shadow-lg p-12 text-center">
          <p className="text-gray-500">No rule groups found</p>
        </div>
      ) : (
        <div className="space-y-4">
          {groups.map((group) => (
            <RuleGroupCard key={group.global_id} group={group} />
          ))}
        </div>
      )}
    </div>
  )
})
