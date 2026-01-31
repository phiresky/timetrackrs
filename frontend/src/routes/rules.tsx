import { createFileRoute } from "@tanstack/react-router";
import { observer } from "mobx-react-lite";
import { useEffect, useState } from "react";
import { makeAutoObservable, runInAction } from "mobx";
import { api } from "../api/client";
import type { TagRuleGroup, TagRule } from "../api/types";

// Simple store for rules page
class RulesStore {
  ruleGroups: TagRuleGroup[] = [];
  isLoading = false;
  error: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  async loadRules() {
    this.isLoading = true;
    this.error = null;

    try {
      const groups = await api.getRuleGroups();
      runInAction(() => {
        this.ruleGroups = groups;
        this.isLoading = false;
      });
    } catch (e) {
      runInAction(() => {
        this.error = e instanceof Error ? e.message : "Unknown error";
        this.isLoading = false;
      });
    }
  }
}

const rulesStore = new RulesStore();

function getRuleDescription(rule: TagRule): string {
  switch (rule.type) {
    case "HasTag":
      return `When tag "${rule.tag}" exists`;
    case "ExactTagValue":
      return `When ${rule.tag} = "${rule.value}"`;
    case "TagValuePrefix":
      return `When ${rule.tag} starts with "${rule.prefix}"`;
    case "TagRegex":
      return `Regex match on ${rule.regexes.map((r) => r.tag).join(", ")}`;
    case "InternalFetcher":
      return `Internal fetcher: ${rule.fetcher_id}`;
    case "ExternalFetcher":
      return `External fetcher: ${rule.fetcher_id}`;
    default:
      return "Unknown rule type";
  }
}

function getRuleOutput(rule: TagRule): string {
  if ("new_tags" in rule && rule.new_tags) {
    return rule.new_tags.map((t) => `${t.tag}:${t.value}`).join(", ");
  }
  if (rule.type === "InternalFetcher" || rule.type === "ExternalFetcher") {
    return `(fetches data from ${rule.fetcher_id})`;
  }
  return "";
}

interface RuleGroupCardProps {
  group: TagRuleGroup;
}

function RuleGroupCard({ group }: RuleGroupCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const data = group.data.data;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
      >
        <div className="flex items-center gap-3">
          <span
            className={`w-2 h-2 rounded-full ${data.enabled ? "bg-green-500" : "bg-gray-400"}`}
          />
          <div className="text-left">
            <h3 className="text-sm font-medium text-gray-900 dark:text-white">
              {data.name}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {data.rules.length} rules • {data.editable ? "Editable" : "Read-only"}
            </p>
          </div>
        </div>
        <svg
          className={`w-5 h-5 text-gray-400 transition-transform ${isExpanded ? "rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {isExpanded && (
        <div className="border-t border-gray-200 dark:border-gray-700">
          {data.description && (
            <p className="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/50">
              {data.description}
            </p>
          )}

          <div className="divide-y divide-gray-200 dark:divide-gray-700">
            {data.rules.map((ruleWithMeta, idx) => (
              <div
                key={idx}
                className={`px-4 py-3 ${!ruleWithMeta.enabled ? "opacity-50" : ""}`}
              >
                <div className="flex items-start gap-2">
                  <span
                    className={`mt-1 w-1.5 h-1.5 rounded-full flex-shrink-0 ${
                      ruleWithMeta.enabled ? "bg-green-500" : "bg-gray-400"
                    }`}
                  />
                  <div className="min-w-0 flex-1">
                    <p className="text-sm text-gray-900 dark:text-white">
                      {getRuleDescription(ruleWithMeta.rule)}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                      → {getRuleOutput(ruleWithMeta.rule) || "(no output tags)"}
                    </p>
                  </div>
                </div>
              </div>
            ))}

            {data.rules.length === 0 && (
              <p className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                No rules in this group
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

const RulesPage = observer(function RulesPage() {
  useEffect(() => {
    rulesStore.loadRules();
  }, []);

  const { ruleGroups, isLoading, error } = rulesStore;

  return (
    <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          Tag Rules
        </h1>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Rules are used to derive additional tags from captured data. They run
          iteratively until no new tags are generated.
        </p>
      </div>

      {error && (
        <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
        </div>
      )}

      {isLoading ? (
        <div className="space-y-4">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-16 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse" />
          ))}
        </div>
      ) : (
        <div className="space-y-4">
          {ruleGroups.map((group) => (
            <RuleGroupCard key={group.global_id} group={group} />
          ))}

          {ruleGroups.length === 0 && (
            <div className="text-center py-12 text-gray-500 dark:text-gray-400">
              <p>No rule groups configured</p>
            </div>
          )}
        </div>
      )}

      {/* Info section */}
      <div className="mt-8 p-4 bg-blue-50 dark:bg-blue-900/30 rounded-lg">
        <h3 className="text-sm font-medium text-blue-800 dark:text-blue-200 mb-2">
          How rules work
        </h3>
        <ul className="text-sm text-blue-700 dark:text-blue-300 space-y-1 list-disc list-inside">
          <li>Rules are applied iteratively to each captured event</li>
          <li>Each rule can match existing tags and add new ones</li>
          <li>Rules continue running until no new tags are added</li>
          <li>External fetchers can enrich data from APIs (YouTube, Wikidata, etc.)</li>
        </ul>
      </div>
    </div>
  );
});

export const Route = createFileRoute("/rules")({
  component: RulesPage,
});
