const API_BASE_URL =
  import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

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
 * Redirect to WorkOS AuthKit for authentication
 */
export function loginWithWorkOS(): void {
  window.location.href = `${API_BASE_URL}/auth/oauth/workos`;
}
