import { createSignal, Show, splitProps, type Component, type JSX } from 'solid-js';
import { useAuth } from '../context';

interface LoginFormProps {
  /**
   * Callback after successful login
   */
  onSuccess?: () => void;
  /**
   * Custom form wrapper component (should render children and accept class)
   */
  FormWrapper?: Component<{ children: JSX.Element; class?: string }>;
  /**
   * Custom input component
   */
  Input?: Component<JSX.InputHTMLAttributes<HTMLInputElement>>;
  /**
   * Custom label component
   */
  Label?: Component<{ for: string; children: JSX.Element }>;
  /**
   * Custom button component
   */
  Button?: Component<JSX.ButtonHTMLAttributes<HTMLButtonElement> & { children: JSX.Element }>;
}

/**
 * Login form component
 *
 * Can be customized with custom UI components or used with defaults.
 */
export const LoginForm: Component<LoginFormProps> = (props) => {
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);

  const auth = useAuth();

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    try {
      await auth.login({
        email: email(),
        password: password(),
      });
      props.onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setIsLoading(false);
    }
  };

  // Default components
  const Input = props.Input || DefaultInput;
  const Label = props.Label || DefaultLabel;
  const Button = props.Button || DefaultButton;

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-md bg-red-50 p-3 border border-red-200">
          <p class="text-sm text-red-800">{error()}</p>
        </div>
      </Show>

      <form onSubmit={handleSubmit} class="space-y-4">
        <div class="space-y-2">
          <Label for="login-email">Email</Label>
          <Input
            id="login-email"
            type="email"
            placeholder="your@email.com"
            value={email()}
            onInput={(e: any) => setEmail(e.currentTarget.value)}
            required
          />
        </div>

        <div class="space-y-2">
          <Label for="login-password">Password</Label>
          <Input
            id="login-password"
            type="password"
            placeholder="Enter your password"
            value={password()}
            onInput={(e: any) => setPassword(e.currentTarget.value)}
            required
          />
        </div>

        <Button type="submit" disabled={isLoading()}>
          {isLoading() ? 'Signing in...' : 'Sign In'}
        </Button>
      </form>
    </div>
  );
};

// Default UI components
const DefaultInput: Component<JSX.InputHTMLAttributes<HTMLInputElement>> = (props) => {
  const [local, rest] = splitProps(props, ['class']);
  return (
    <input
      {...rest}
      class={`flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 ${local.class || ''}`}
    />
  );
};

const DefaultLabel: Component<{ for: string; children: JSX.Element }> = (props) => (
  <label for={props.for} class="text-sm font-medium leading-none">
    {props.children}
  </label>
);

const DefaultButton: Component<JSX.ButtonHTMLAttributes<HTMLButtonElement> & { children: JSX.Element }> = (props) => {
  const [local, rest] = splitProps(props, ['class', 'children']);
  return (
    <button
      {...rest}
      class={`w-full inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:pointer-events-none ${local.class || ''}`}
    >
      {local.children}
    </button>
  );
};
