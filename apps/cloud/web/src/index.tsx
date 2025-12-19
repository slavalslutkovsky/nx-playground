/* @refresh reload */
import { render } from "solid-js/web";
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/solid-router";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { Layout } from "./components/layout";
import Dashboard from "./pages/dashboard";
import TcoCalculator from "./pages/tco-calculator";
import CncfTools from "./pages/cncf-tools";
import CncfLandscape from "./pages/cncf-landscape";
import Compare from "./pages/compare";
import PriceFinder from "./pages/price-finder";
import "./index.css";
import "solid-devtools";

// Create a query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
    },
  },
});

// Layout wrapper component
function AppLayout() {
  return (
    <Layout>
      <Outlet />
    </Layout>
  );
}

// Create the root route
const rootRoute = createRootRoute({
  component: AppLayout,
});

// Create routes
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: Dashboard,
});

const tcoRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/tco",
  component: TcoCalculator,
});

const toolsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/tools",
  component: CncfTools,
});

const landscapeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/landscape",
  component: CncfLandscape,
});

const compareRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/compare",
  component: Compare,
});

const finderRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/finder",
  component: PriceFinder,
});

// Create the route tree
const routeTree = rootRoute.addChildren([indexRoute, tcoRoute, toolsRoute, landscapeRoute, compareRoute, finderRoute]);

// Create the router
const router = createRouter({ routeTree });

// Render the app
const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html?"
  );
}

if (root) {
  render(
    () => (
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    ),
    root
  );
}
