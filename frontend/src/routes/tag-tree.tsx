import { createRoute } from '@tanstack/react-router'
import { rootRoute } from './__root'
import { TagTree } from '../pages/TagTree'

export const tagTreeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tag-tree',
  component: TagTree,
})
