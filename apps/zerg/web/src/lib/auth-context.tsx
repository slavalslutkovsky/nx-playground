import {
  createMutation,
  createQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import { createContext, type ParentComponent, useContext } from 'solid-js';
import type { LoginRequest, RegisterRequest, UserResponse } from './auth-api';
import * as authApi from './auth-api';

interface AuthContextValue {
  user: () => UserResponse | null | undefined;
  isLoading: () => boolean;
  isAuthenticated: () => boolean;
  login: (data: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => void;
}

const AuthContext = createContext<AuthContextValue>();

export const AuthProvider: ParentComponent = (props) => {
  const queryClient = useQueryClient();

  // Query for current user
  const userQuery = createQuery(() => ({
    queryKey: ['currentUser'],
    queryFn: authApi.getCurrentUser,
    retry: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
    // Don't throw errors, just return undefined
    throwOnError: false,
  }));

  // Login mutation
  const loginMutation = createMutation(() => ({
    mutationFn: authApi.login,
    onSuccess: (data) => {
      // Update user in cache
      queryClient.setQueryData(['currentUser'], data.user);
    },
  }));

  // Register mutation
  const registerMutation = createMutation(() => ({
    mutationFn: authApi.register,
    onSuccess: (data) => {
      // Update user in cache
      queryClient.setQueryData(['currentUser'], data.user);
    },
  }));

  // Logout mutation
  const logoutMutation = createMutation(() => ({
    mutationFn: authApi.logout,
    onSuccess: () => {
      // Clear user from cache
      queryClient.setQueryData(['currentUser'], null);
      // Redirect to login
      window.location.href = '/login';
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

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
