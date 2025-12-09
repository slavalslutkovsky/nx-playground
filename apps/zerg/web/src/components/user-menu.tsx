import { Show, createSignal } from 'solid-js';
import { Button } from './ui/button';
import { useAuth } from '../lib/auth-context';

export function UserMenu() {
  const auth = useAuth();
  const [isOpen, setIsOpen] = createSignal(false);

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map((n) => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  const handleLogout = () => {
    setIsOpen(false);
    auth.logout();
  };

  return (
    <Show when={auth.isAuthenticated() && auth.user()}>
      {(user) => (
        <div class="relative">
          <Button
            variant="ghost"
            class="flex items-center gap-2"
            onClick={() => setIsOpen(!isOpen())}
          >
            <div class="h-8 w-8 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-medium">
              <Show when={user().avatar_url} fallback={getInitials(user().name)}>
                <img
                  src={user().avatar_url}
                  alt={user().name}
                  class="h-full w-full rounded-full object-cover"
                />
              </Show>
            </div>
            <Show when={window.innerWidth > 640}>
              <span class="text-sm font-medium">{user().name}</span>
            </Show>
          </Button>

          <Show when={isOpen()}>
            <div
              class="absolute right-0 mt-2 w-56 rounded-md border bg-card shadow-lg z-50"
              onClick={() => setIsOpen(false)}
            >
              <div class="border-b px-3 py-2">
                <p class="text-sm font-semibold">{user().name}</p>
                <p class="text-xs text-muted-foreground">{user().email}</p>
              </div>
              <div class="p-1">
                <button
                  onClick={handleLogout}
                  class="w-full text-left px-3 py-2 text-sm text-red-600 hover:bg-accent rounded-sm"
                >
                  Sign out
                </button>
              </div>
            </div>
          </Show>

          {/* Backdrop to close menu when clicking outside */}
          <Show when={isOpen()}>
            <div
              class="fixed inset-0 z-40"
              onClick={() => setIsOpen(false)}
            />
          </Show>
        </div>
      )}
    </Show>
  );
}
