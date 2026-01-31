import { createRouter } from '@tanstack/react-router'
import { rootRoute } from './__root'
import { indexRoute } from './index'
import { timelineRoute } from './timeline'
import { tagTreeRoute } from './tag-tree'
import { plotRoute } from './plot'
import { ruleEditorRoute } from './rule-editor'

const routeTree = rootRoute.addChildren([
  indexRoute,
  timelineRoute,
  tagTreeRoute,
  plotRoute,
  ruleEditorRoute,
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
