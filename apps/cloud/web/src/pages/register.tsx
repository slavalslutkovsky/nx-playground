import { Link, useNavigate } from '@tanstack/solid-router';
import { RegisterForm } from '@nx-playground/auth-solid';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '~/components/ui/card';

export default function RegisterPage() {
  const navigate = useNavigate();

  const handleSuccess = () => {
    navigate({ to: '/' });
  };

  return (
    <div class="flex min-h-[calc(100vh-12rem)] items-center justify-center p-4">
      <Card class="w-full max-w-md">
        <CardHeader>
          <CardTitle class="text-2xl text-center">Create Account</CardTitle>
          <CardDescription class="text-center">
            Sign up to start optimizing your cloud costs
          </CardDescription>
        </CardHeader>

        <CardContent class="space-y-4">
          <RegisterForm onSuccess={handleSuccess} />
        </CardContent>

        <CardFooter class="flex justify-center">
          <p class="text-sm text-muted-foreground">
            Already have an account?{' '}
            <Link to="/login" class="text-primary hover:underline">
              Sign in
            </Link>
          </p>
        </CardFooter>
      </Card>
    </div>
  );
}
