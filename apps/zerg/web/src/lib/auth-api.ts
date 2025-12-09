const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

export interface UserResponse {
  id: string;
  email: string;
  name: string;
  roles: string[];
  email_verified: boolean;
  created_at: string;
  updated_at: string;
  avatar_url?: string;
  last_login_at?: string;
}

export interface LoginResponse {
  user: UserResponse;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

/**
 * Register a new user
 */
export async function register(data: RegisterRequest): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    credentials: 'include', // Important: include cookies
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error?.message || 'Registration failed');
  }

  return response.json();
}

/**
 * Login with email and password
 */
export async function login(data: LoginRequest): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    credentials: 'include', // Important: include cookies
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error?.message || 'Login failed');
  }

  return response.json();
}

/**
 * Logout current user
 */
export async function logout(): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/auth/logout`, {
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
export async function getCurrentUser(): Promise<UserResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/me`, {
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
 * Redirect to Google OAuth flow
 */
export function loginWithGoogle(): void {
  window.location.href = `${API_BASE_URL}/auth/oauth/google`;
}

/**
 * Redirect to GitHub OAuth flow
 */
export function loginWithGithub(): void {
  window.location.href = `${API_BASE_URL}/auth/oauth/github`;
}
