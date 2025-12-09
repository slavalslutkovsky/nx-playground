import './index.css';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
  Navigate,
} from '@tanstack/solid-router';
import { render } from 'solid-js/web';
import 'solid-devtools';

import { AuthProvider } from './lib/auth-context';
import { ProtectedRoute } from './components/protected-route';
import { UserMenu } from './components/user-menu';
import { TaskDetailPage } from './pages/task-detail';
import { TasksListPage } from './pages/tasks-list';
import { LoginPage } from './pages/login';
import { RegisterPage } from './pages/register';

const queryClient = new QueryClient();

// Layout component with navigation
function Layout() {
  return (
    <div>
      <nav class="border-b bg-white shadow-sm">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex justify-between h-16 items-center">
            <div class="flex items-center">
              <h1 class="text-xl font-bold">Zerg Tasks</h1>
            </div>
            <div class="flex items-center">
              <UserMenu />
            </div>
          </div>
        </div>
      </nav>
      <main>
        <Outlet />
      </main>
    </div>
  );
}

// Create a root route with layout
const rootRoute = createRootRoute({
  component: Layout,
});

// Index route - redirect to tasks
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => <Navigate to="/tasks" />,
});

// Public routes
const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
});

const registerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/register',
  component: RegisterPage,
});

// Protected routes
const tasksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks',
  component: () => (
    <ProtectedRoute>
      <TasksListPage />
    </ProtectedRoute>
  ),
});

const taskDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks/$id',
  component: () => (
    <ProtectedRoute>
      <TaskDetailPage />
    </ProtectedRoute>
  ),
});

// Build route tree
const routeTree = rootRoute.addChildren([
  indexRoute,
  loginRoute,
  registerRoute,
  tasksRoute,
  taskDetailRoute,
]);

// Create router
const router = createRouter({ routeTree });

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  );
}

if (root) {
  render(
    () => (
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <RouterProvider router={router} />
        </AuthProvider>
      </QueryClientProvider>
    ),
    root,
  );
}
