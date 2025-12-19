import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  UserResponse,
  OAuthProvider,
} from './types';

/**
 * Create an auth API client with the given base URL
 */
export function createAuthApi(baseUrl: string) {
  /**
   * Register a new user
   */
  async function register(data: RegisterRequest): Promise<LoginResponse> {
    const response = await fetch(`${baseUrl}/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.error?.message || error.message || 'Registration failed');
    }

    return response.json();
  }

  /**
   * Login with email and password
   */
  async function login(data: LoginRequest): Promise<LoginResponse> {
    const response = await fetch(`${baseUrl}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.error?.message || error.message || 'Login failed');
    }

    return response.json();
  }

  /**
   * Logout current user
   */
  async function logout(): Promise<void> {
    const response = await fetch(`${baseUrl}/auth/logout`, {
      method: 'POST',
      credentials: 'include',
    });

    if (!response.ok) {
      throw new Error('Logout failed');
    }
  }

  /**
   * Get current authenticated user
   */
  async function getCurrentUser(): Promise<UserResponse> {
    const response = await fetch(`${baseUrl}/auth/me`, {
      credentials: 'include',
    });

    if (!response.ok) {
      if (response.status === 401) {
        throw new Error('Not authenticated');
      }
      throw new Error('Failed to get user');
    }

    return response.json();
  }

  /**
   * Redirect to OAuth provider login
   */
  function loginWithOAuth(provider: OAuthProvider): void {
    window.location.href = `${baseUrl}/auth/oauth/${provider}`;
  }

  return {
    register,
    login,
    logout,
    getCurrentUser,
    loginWithOAuth,
  };
}

export type AuthApi = ReturnType<typeof createAuthApi>;
