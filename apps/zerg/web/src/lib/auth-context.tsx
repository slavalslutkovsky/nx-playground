import {
  createMutation,
  createQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import { createContext, type ParentComponent, useContext } from 'solid-js';
import type { UserResponse } from './auth-api';
import * as authApi from './auth-api';

interface AuthContextValue {
  user: () => UserResponse | null | undefined;
  isLoading: () => boolean;
  isAuthenticated: () => boolean;
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
