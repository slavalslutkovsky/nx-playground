import { createMemo, createSignal, Show, splitProps, type Component, type JSX } from 'solid-js';
import { useAuth } from '../context';

interface RegisterFormProps {
  /**
   * Callback after successful registration
   */
  onSuccess?: () => void;
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
 * Registration form component with password validation
 */
export const RegisterForm: Component<RegisterFormProps> = (props) => {
  const [name, setName] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [error, setError] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);

  const auth = useAuth();

  // Password validation
  const passwordRequirements = createMemo(() => ({
    length: password().length >= 8,
    uppercase: /[A-Z]/.test(password()),
    lowercase: /[a-z]/.test(password()),
    digit: /\d/.test(password()),
    special: /[!@#$%^&*()_+\-=[\]{}|;:,.<>?]/.test(password()),
  }));

  const isPasswordValid = createMemo(() => {
    const reqs = passwordRequirements();
    return reqs.length && reqs.uppercase && reqs.lowercase && reqs.digit && reqs.special;
  });

  const passwordsMatch = createMemo(() => {
    return password() === confirmPassword() && password().length > 0;
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError('');

    if (!isPasswordValid()) {
      setError('Password does not meet requirements');
      return;
    }

    if (!passwordsMatch()) {
      setError('Passwords do not match');
      return;
    }

    setIsLoading(true);

    try {
      await auth.register({
        name: name(),
        email: email(),
        password: password(),
      });
      props.onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Registration failed');
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
          <Label for="register-name">Name</Label>
          <Input
            id="register-name"
            type="text"
            placeholder="Your name"
            value={name()}
            onInput={(e: any) => setName(e.currentTarget.value)}
            required
          />
        </div>

        <div class="space-y-2">
          <Label for="register-email">Email</Label>
          <Input
            id="register-email"
            type="email"
            placeholder="your@email.com"
            value={email()}
            onInput={(e: any) => setEmail(e.currentTarget.value)}
            required
          />
        </div>

        <div class="space-y-2">
          <Label for="register-password">Password</Label>
          <Input
            id="register-password"
            type="password"
            placeholder="Create a password"
            value={password()}
            onInput={(e: any) => setPassword(e.currentTarget.value)}
            required
          />
        </div>

        <Show when={password().length > 0}>
          <PasswordRequirements requirements={passwordRequirements()} />
        </Show>

        <div class="space-y-2">
          <Label for="register-confirm-password">Confirm Password</Label>
          <Input
            id="register-confirm-password"
            type="password"
            placeholder="Confirm your password"
            value={confirmPassword()}
            onInput={(e: any) => setConfirmPassword(e.currentTarget.value)}
            required
          />
          <Show when={confirmPassword().length > 0 && !passwordsMatch()}>
            <p class="text-sm text-red-600">Passwords do not match</p>
          </Show>
        </div>

        <Button type="submit" disabled={!isPasswordValid() || !passwordsMatch() || isLoading()}>
          {isLoading() ? 'Creating account...' : 'Create Account'}
        </Button>
      </form>
    </div>
  );
};

// Password requirements display
interface PasswordRequirementsProps {
  requirements: {
    length: boolean;
    uppercase: boolean;
    lowercase: boolean;
    digit: boolean;
    special: boolean;
  };
}

const PasswordRequirements: Component<PasswordRequirementsProps> = (props) => {
  const RequirementItem: Component<{ met: boolean; text: string }> = (item) => (
    <li class={item.met ? 'text-green-600' : 'text-muted-foreground'}>
      <span class="mr-2">{item.met ? '\u2713' : '\u25CB'}</span>
      {item.text}
    </li>
  );

  return (
    <div class="rounded-md bg-muted p-3 space-y-2">
      <p class="text-sm font-semibold">Password requirements:</p>
      <ul class="space-y-1 text-sm">
        <RequirementItem met={props.requirements.length} text="At least 8 characters" />
        <RequirementItem met={props.requirements.uppercase} text="One uppercase letter" />
        <RequirementItem met={props.requirements.lowercase} text="One lowercase letter" />
        <RequirementItem met={props.requirements.digit} text="One number" />
        <RequirementItem met={props.requirements.special} text="One special character" />
      </ul>
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
