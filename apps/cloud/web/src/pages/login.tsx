import { Link, useNavigate } from '@tanstack/solid-router';
import { LoginForm, SocialLogin } from '@nx-playground/auth-solid';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '~/components/ui/card';

export default function LoginPage() {
  const navigate = useNavigate();

  const handleSuccess = () => {
    navigate({ to: '/' });
  };

  return (
    <div class="flex min-h-[calc(100vh-12rem)] items-center justify-center p-4">
      <Card class="w-full max-w-md">
        <CardHeader>
          <CardTitle class="text-2xl text-center">Sign In</CardTitle>
          <CardDescription class="text-center">
            Enter your credentials to access Cloud Cost Optimizer
          </CardDescription>
        </CardHeader>

        <CardContent class="space-y-4">
          <LoginForm onSuccess={handleSuccess} />
          <SocialLogin />
        </CardContent>

        <CardFooter class="flex justify-center">
          <p class="text-sm text-muted-foreground">
            Don't have an account?{' '}
            <Link to="/register" class="text-primary hover:underline">
              Sign up
            </Link>
          </p>
        </CardFooter>
      </Card>
    </div>
  );
}
