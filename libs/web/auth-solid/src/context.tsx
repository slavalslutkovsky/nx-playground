import {
  createMutation,
  createQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import {
  createContext,
  type ParentComponent,
  useContext,
} from 'solid-js';
import type { AuthContextValue, AuthConfig, LoginRequest, RegisterRequest } from './types';
import { createAuthApi, type AuthApi } from './api';

const AuthContext = createContext<AuthContextValue>();

// Store the API instance and config globally within the module
let authApi: AuthApi;
let authConfig: AuthConfig;

/**
 * Get the auth API instance
 */
export function getAuthApi(): AuthApi {
  if (!authApi) {
    throw new Error('Auth not initialized. Wrap your app with AuthProvider.');
  }
  return authApi;
}

/**
 * Get the auth configuration
 */
export function getAuthConfig(): AuthConfig {
  if (!authConfig) {
    throw new Error('Auth not initialized. Wrap your app with AuthProvider.');
  }
  return authConfig;
}

interface AuthProviderProps {
  config: AuthConfig;
}

/**
 * Auth provider component
 *
 * Wrap your app with this provider to enable authentication.
 *
 * @example
 * ```tsx
 * <AuthProvider config={{ apiBaseUrl: 'http://localhost:8080/api' }}>
 *   <App />
 * </AuthProvider>
 * ```
 */
export const AuthProvider: ParentComponent<AuthProviderProps> = (props) => {
  // Initialize the API
  authConfig = {
    loginRedirectPath: '/',
    loginPath: '/login',
    oauthProviders: ['google', 'github'],
    ...props.config,
  };
  authApi = createAuthApi(authConfig.apiBaseUrl);

  const queryClient = useQueryClient();

  // Query for current user
  const userQuery = createQuery(() => ({
    queryKey: ['currentUser'],
    queryFn: () => authApi.getCurrentUser(),
    retry: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
    throwOnError: false,
  }));

  // Login mutation
  const loginMutation = createMutation(() => ({
    mutationFn: (data: LoginRequest) => authApi.login(data),
    onSuccess: (data) => {
      queryClient.setQueryData(['currentUser'], data.user);
    },
  }));

  // Register mutation
  const registerMutation = createMutation(() => ({
    mutationFn: (data: RegisterRequest) => authApi.register(data),
    onSuccess: (data) => {
      queryClient.setQueryData(['currentUser'], data.user);
    },
  }));

  // Logout mutation
  const logoutMutation = createMutation(() => ({
    mutationFn: () => authApi.logout(),
    onSuccess: () => {
      queryClient.setQueryData(['currentUser'], null);
      window.location.href = authConfig.loginPath || '/login';
    },
  }));

  const login = async (data: LoginRequest) => {
    await loginMutation.mutateAsync(data);
  };

  const register = async (data: RegisterRequest) => {
    await registerMutation.mutateAsync(data);
  };

  const logout = async () => {
    await logoutMutation.mutateAsync();
  };

  const checkAuth = () => {
    userQuery.refetch();
  };

  const isAuthenticated = () => {
    return !!userQuery.data && !userQuery.isError;
  };

  const isLoading = () => {
    return userQuery.isLoading || userQuery.isFetching;
  };

  const value: AuthContextValue = {
    user: () => userQuery.data,
    isLoading,
    isAuthenticated,
    login,
    register,
    logout,
    checkAuth,
  };

  return (
    <AuthContext.Provider value={value}>{props.children}</AuthContext.Provider>
  );
};

/**
 * Hook to access auth context
 *
 * @throws Error if used outside of AuthProvider
 */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
