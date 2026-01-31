import { createRootRoute, Outlet } from '@tanstack/react-router'
import { Layout } from '../components/Layout'

export const rootRoute = createRootRoute({
  component: () => (
    <Layout>
      <Outlet />
    </Layout>
  ),
})
