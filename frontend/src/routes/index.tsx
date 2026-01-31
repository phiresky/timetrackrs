import { createRoute } from '@tanstack/react-router'
import { rootRoute } from './__root'
import { Dashboard } from '../pages/Dashboard'

export const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: Dashboard,
})
