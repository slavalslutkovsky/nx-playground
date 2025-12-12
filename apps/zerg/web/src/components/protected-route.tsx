import { useNavigate } from '@tanstack/solid-router';
import { createEffect, type ParentComponent, Show } from 'solid-js';
import { useAuth } from '../lib/auth-context';

export const ProtectedRoute: ParentComponent = (props) => {
  const auth = useAuth();
  const navigate = useNavigate();

  createEffect(() => {
    // If not loading and not authenticated, redirect to login
    if (!auth.isLoading() && !auth.isAuthenticated()) {
      navigate({ to: '/login' });
    }
  });

  return (
    <Show
      when={!auth.isLoading()}
      fallback={
        <div class="flex items-center justify-center min-h-screen">
          <div class="text-center">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
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
