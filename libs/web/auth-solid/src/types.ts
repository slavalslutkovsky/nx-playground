/**
 * User response from the API
 */
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
 * Login response containing user data
 */
export interface LoginResponse {
  user: UserResponse;
}

/**
 * Registration request data
 */
export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

/**
 * Login request data
 */
export interface LoginRequest {
  email: string;
  password: string;
}

/**
 * OAuth providers supported
 */
export type OAuthProvider = 'google' | 'github';

/**
 * Auth context value interface
 */
export interface AuthContextValue {
  user: () => UserResponse | null | undefined;
  isLoading: () => boolean;
  isAuthenticated: () => boolean;
  login: (data: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => void;
}

/**
 * Auth configuration options
 */
export interface AuthConfig {
  /**
   * Base URL for the API (e.g., 'http://localhost:8080/api')
   */
  apiBaseUrl: string;

  /**
   * Path to redirect after successful login
   * @default '/'
   */
  loginRedirectPath?: string;

  /**
   * Path to the login page
   * @default '/login'
   */
  loginPath?: string;

  /**
   * Enable OAuth providers
   * @default ['google', 'github']
   */
  oauthProviders?: OAuthProvider[];
}
