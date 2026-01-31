import { createRootRoute, Link, Outlet } from "@tanstack/react-router";
import { EventDetail, CategorizationModal } from "../components";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <nav className="bg-white dark:bg-gray-800 shadow-sm sticky top-0 z-40">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-14">
            <div className="flex items-center">
              <Link to="/" className="flex items-center gap-2">
                <svg
                  className="w-6 h-6 text-indigo-600"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                <span className="font-semibold text-gray-900 dark:text-white">
                  timetrackrs
                </span>
              </Link>
            </div>
            <div className="flex items-center space-x-1">
              <Link
                to="/"
                className="px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors [&.active]:bg-indigo-50 [&.active]:text-indigo-700 dark:[&.active]:bg-indigo-900/50 dark:[&.active]:text-indigo-300"
              >
                Timeline
              </Link>
              <Link
                to="/stats"
                className="px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors [&.active]:bg-indigo-50 [&.active]:text-indigo-700 dark:[&.active]:bg-indigo-900/50 dark:[&.active]:text-indigo-300"
              >
                Statistics
              </Link>
              <Link
                to="/rules"
                className="px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors [&.active]:bg-indigo-50 [&.active]:text-indigo-700 dark:[&.active]:bg-indigo-900/50 dark:[&.active]:text-indigo-300"
              >
                Rules
              </Link>
            </div>
          </div>
        </div>
      </nav>
      <main>
        <Outlet />
      </main>
      <EventDetail />
      <CategorizationModal />
    </div>
  );
}
