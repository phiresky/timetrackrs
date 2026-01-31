import { createRoute } from '@tanstack/react-router'
import { rootRoute } from './__root'
import { Timeline } from '../pages/Timeline'

export const timelineRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/timeline',
  component: Timeline,
})
