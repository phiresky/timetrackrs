import { createRoute } from '@tanstack/react-router'
import { rootRoute } from './__root'
import { Plot } from '../pages/Plot'

export const plotRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/plot',
  component: Plot,
})
