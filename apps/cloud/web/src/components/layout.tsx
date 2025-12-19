import { type Component, type JSX, Show } from "solid-js";
import { Link } from "@tanstack/solid-router";
import { useAuth, UserMenu } from "@nx-playground/auth-solid";
import { Button } from "~/components/ui/button";

interface LayoutProps {
  children: JSX.Element;
}

const Layout: Component<LayoutProps> = (props) => {
  const auth = useAuth();

  return (
    <div class="min-h-screen bg-background">
      {/* Navigation */}
      <nav class="border-b bg-card">
        <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div class="flex h-16 items-center justify-between">
            <div class="flex items-center">
              <Link to="/" class="flex items-center gap-2">
                <svg
                  class="h-8 w-8 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z"
                  />
                </svg>
                <span class="text-xl font-bold">Cloud Cost Optimizer</span>
              </Link>
            </div>
            <div class="flex items-center gap-6">
              <Link
                to="/"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                Dashboard
              </Link>
              <Link
                to="/tco"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                TCO Calculator
              </Link>
              <Link
                to="/tools"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                CNCF Tools
              </Link>
              <Link
                to="/landscape"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                Landscape
              </Link>
              <Link
                to="/compare"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                Compare
              </Link>
              <Link
                to="/finder"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                Price Finder
              </Link>
              <Link
                to="/chat"
                class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                activeProps={{ class: "text-foreground" }}
              >
                AI Assistant
              </Link>
              <div class="ml-4 border-l pl-4">
                <Show
                  when={auth.user()}
                  fallback={
                    <Link to="/login">
                      <Button variant="outline" size="sm">
                        Sign In
                      </Button>
                    </Link>
                  }
                >
                  <UserMenu />
                </Show>
              </div>
            </div>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
        {props.children}
      </main>

      {/* Footer */}
      <footer class="border-t bg-card mt-auto">
        <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <div class="flex justify-between items-center text-sm text-muted-foreground">
            <p>Cloud Cost Optimizer - Compare AWS, Azure, GCP pricing</p>
            <p>Prices are indicative and may vary by region and usage</p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export { Layout };
