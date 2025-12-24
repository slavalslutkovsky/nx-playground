import { createSignal, Show } from 'solid-js';
import { Button } from '../components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '../components/ui/card';

export function LoginPage() {
  const [isLoading, setIsLoading] = createSignal(false);

  const handleSignIn = () => {
    setIsLoading(true);
    // Redirect to WorkOS AuthKit via our backend
    const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
    window.location.href = `${apiUrl}/api/auth/oauth/workos`;
  };

  return (
    <div class="flex min-h-screen items-center justify-center p-4">
      <Card class="w-full max-w-md">
        <CardHeader class="text-center">
          <CardTitle class="text-2xl">Welcome</CardTitle>
          <CardDescription>
            Sign in to access your account
          </CardDescription>
        </CardHeader>

        <CardContent class="space-y-4">
          <Button
            type="button"
            class="w-full"
            size="lg"
            onClick={handleSignIn}
            disabled={isLoading()}
          >
            <Show when={isLoading()} fallback="Sign in">
              <div class="flex items-center gap-2">
                <div class="h-4 w-4 animate-spin rounded-full border-2 border-solid border-current border-r-transparent" />
                <span>Redirecting...</span>
              </div>
            </Show>
          </Button>

          <p class="text-center text-sm text-muted-foreground">
            Secure authentication powered by WorkOS
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
