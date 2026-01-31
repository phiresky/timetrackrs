import { Link, useRouterState } from '@tanstack/react-router'

const navItems = [
  { to: '/', label: 'Dashboard' },
  { to: '/timeline', label: 'Timeline' },
  { to: '/tag-tree', label: 'Tag Tree' },
  { to: '/plot', label: 'Plot' },
  { to: '/rule-editor', label: 'Rule Editor' },
] as const

export function Navbar() {
  const routerState = useRouterState()
  const currentPath = routerState.location.pathname

  return (
    <nav className="bg-indigo-600 shadow-lg">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center">
            <Link to="/" className="flex-shrink-0">
              <span className="text-white text-xl font-bold">Timetrackrs</span>
            </Link>
            <div className="hidden md:block ml-10">
              <div className="flex items-baseline space-x-4">
                {navItems.map((item) => {
                  const isActive = currentPath === item.to
                  return (
                    <Link
                      key={item.to}
                      to={item.to}
                      className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                        isActive
                          ? 'bg-indigo-700 text-white'
                          : 'text-indigo-200 hover:bg-indigo-500 hover:text-white'
                      }`}
                    >
                      {item.label}
                    </Link>
                  )
                })}
              </div>
            </div>
          </div>
        </div>
      </div>
      {/* Mobile menu */}
      <div className="md:hidden">
        <div className="px-2 pt-2 pb-3 space-y-1 sm:px-3">
          {navItems.map((item) => {
            const isActive = currentPath === item.to
            return (
              <Link
                key={item.to}
                to={item.to}
                className={`block px-3 py-2 rounded-md text-base font-medium ${
                  isActive
                    ? 'bg-indigo-700 text-white'
                    : 'text-indigo-200 hover:bg-indigo-500 hover:text-white'
                }`}
              >
                {item.label}
              </Link>
            )
          })}
        </div>
      </div>
    </nav>
  )
}
