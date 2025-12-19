import { useNavigate } from '@tanstack/solid-router';
import { createEffect, type ParentComponent, Show } from 'solid-js';
import { useAuth, getAuthConfig } from '../context';

/**
 * Protected route wrapper component
 *
 * Redirects to login page if user is not authenticated.
 *
 * @example
 * ```tsx
 * <ProtectedRoute>
 *   <Dashboard />
 * </ProtectedRoute>
 * ```
 */
export const ProtectedRoute: ParentComponent = (props) => {
  const auth = useAuth();
  const navigate = useNavigate();

  createEffect(() => {
    if (!auth.isLoading() && !auth.isAuthenticated()) {
      const config = getAuthConfig();
      navigate({ to: config.loginPath || '/login' });
    }
  });

  return (
    <Show
      when={!auth.isLoading()}
      fallback={
        <div class="flex items-center justify-center min-h-screen">
          <div class="text-center">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
            <p class="mt-4 text-gray-600">Loading...</p>
          </div>
        </div>
      }
    >
      <Show when={auth.isAuthenticated()} fallback={null}>
        {props.children}
      </Show>
    </Show>
  );
};
