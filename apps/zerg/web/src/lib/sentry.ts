import * as Sentry from '@sentry/browser';

export function initSentry() {
  // Only initialize in production or if DSN is provided
  const dsn = import.meta.env.VITE_SENTRY_DSN;

  if (!dsn) {
    console.warn('Sentry DSN not configured. Error tracking disabled.');
    return;
  }

  Sentry.init({
    dsn,
    environment: import.meta.env.MODE,
    release: import.meta.env.VITE_APP_VERSION || 'dev',

    // Performance Monitoring
    tracesSampleRate: import.meta.env.PROD ? 0.1 : 1.0,

    // Session Replay (optional)
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,

    // Integrations
    integrations: [
      Sentry.browserTracingIntegration(),
      Sentry.replayIntegration({
        maskAllText: false,
        blockAllMedia: false,
      }),
    ],

    // Filter out noisy errors
    ignoreErrors: [
      // Browser extensions
      /extensions\//i,
      /^chrome:\/\//i,
      // Network errors that are expected
      'Network request failed',
      'Failed to fetch',
      'Load failed',
      // User aborted requests
      'AbortError',
    ],

    // Before sending, you can modify or filter events
    beforeSend(event, hint) {
      // Don't send events in development unless explicitly enabled
      if (import.meta.env.DEV && !import.meta.env.VITE_SENTRY_DEV_ENABLED) {
        console.log('[Sentry] Event captured (dev mode):', event);
        return null;
      }
      return event;
    },
  });
}

// Helper to capture errors manually
export function captureError(error: Error, context?: Record<string, unknown>) {
  Sentry.captureException(error, {
    extra: context,
  });
}

// Helper to capture messages
export function captureMessage(
  message: string,
  level: Sentry.SeverityLevel = 'info',
) {
  Sentry.captureMessage(message, level);
}

// Helper to set user context
export function setUser(
  user: { id: string; email?: string; username?: string } | null,
) {
  if (user) {
    Sentry.setUser(user);
  } else {
    Sentry.setUser(null);
  }
}

// Helper to add breadcrumb
export function addBreadcrumb(
  message: string,
  category: string,
  data?: Record<string, unknown>,
) {
  Sentry.addBreadcrumb({
    message,
    category,
    data,
    level: 'info',
  });
}

// Re-export Sentry for direct access if needed
export { Sentry };
