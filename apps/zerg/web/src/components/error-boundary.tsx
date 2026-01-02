import type { JSX, ParentProps } from 'solid-js';
import { ErrorBoundary as SolidErrorBoundary } from 'solid-js';
import { captureError } from '../lib/sentry';

interface ErrorFallbackProps {
  error: Error;
  reset: () => void;
}

function DefaultErrorFallback(props: ErrorFallbackProps) {
  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-100">
      <div class="max-w-md w-full bg-white shadow-lg rounded-lg p-6">
        <div class="flex items-center justify-center w-12 h-12 mx-auto bg-red-100 rounded-full">
          <svg
            class="w-6 h-6 text-red-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        </div>
        <h2 class="mt-4 text-xl font-semibold text-center text-gray-900">
          Something went wrong
        </h2>
        <p class="mt-2 text-sm text-center text-gray-600">
          We've been notified and are working on a fix.
        </p>
        {import.meta.env.DEV && (
          <details class="mt-4 p-3 bg-gray-50 rounded text-sm">
            <summary class="cursor-pointer font-medium text-gray-700">
              Error Details
            </summary>
            <pre class="mt-2 overflow-auto text-xs text-red-600">
              {props.error.message}
              {'\n\n'}
              {props.error.stack}
            </pre>
          </details>
        )}
        <div class="mt-6 flex gap-3 justify-center">
          <button
            type="button"
            onClick={props.reset}
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Try Again
          </button>
          <button
            type="button"
            onClick={() => (window.location.href = '/')}
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
          >
            Go Home
          </button>
        </div>
      </div>
    </div>
  );
}

interface SentryErrorBoundaryProps extends ParentProps {
  fallback?: (props: ErrorFallbackProps) => JSX.Element;
}

export function SentryErrorBoundary(props: SentryErrorBoundaryProps) {
  const handleError = (error: Error, reset: () => void) => {
    // Report to Sentry
    captureError(error, {
      componentStack: 'ErrorBoundary',
      url: window.location.href,
    });

    const FallbackComponent = props.fallback || DefaultErrorFallback;
    return <FallbackComponent error={error} reset={reset} />;
  };

  return (
    <SolidErrorBoundary fallback={(err, reset) => handleError(err, reset)}>
      {props.children}
    </SolidErrorBoundary>
  );
}
