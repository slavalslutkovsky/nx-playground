import './index.css';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from '@tanstack/solid-router';
import { render } from 'solid-js/web';
import 'solid-devtools';

import { TaskDetailPage } from './pages/task-detail';
import { TasksListPage } from './pages/tasks-list';

const queryClient = new QueryClient();

// Create a root route
const rootRoute = createRootRoute({
  component: () => <Outlet />,
});

// Create routes
const tasksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks',
  component: TasksListPage,
});

const taskDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks/$id',
  component: TaskDetailPage,
});

// Build route tree
const routeTree = rootRoute.addChildren([tasksRoute, taskDetailRoute]);

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
        <RouterProvider router={router} />
      </QueryClientProvider>
    ),
    root,
  );
}
