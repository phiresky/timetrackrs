import { createRoute } from '@tanstack/react-router'
import { rootRoute } from './__root'
import { RuleEditor } from '../pages/RuleEditor'

export const ruleEditorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/rule-editor',
  component: RuleEditor,
})
