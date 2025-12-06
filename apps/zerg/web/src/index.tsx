import './index.css';
import { render } from 'solid-js/web';
import {
  RouterProvider,
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
} from '@tanstack/solid-router';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import 'solid-devtools';

import { TasksListPage } from './pages/tasks-list';
import { TaskDetailPage } from './pages/task-detail';

const queryClient = new QueryClient();

// Create root route
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

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  ),
  root!
);
