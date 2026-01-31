import type { ReactNode } from 'react'
import { Navbar } from './Navbar'
import { ProgressPopup } from './ProgressPopup'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen bg-gray-100">
      <Navbar />
      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">{children}</main>
      <ProgressPopup />
    </div>
  )
}
