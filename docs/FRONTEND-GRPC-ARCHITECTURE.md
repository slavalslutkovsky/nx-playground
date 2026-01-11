# Frontend/Fullstack gRPC Architecture Guide

Architecture patterns for Node.js frontend/fullstack teams consuming Rust gRPC backends in Kubernetes.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Recommended Pattern: BFF with Server Actions](#recommended-pattern-bff-with-server-actions)
3. [Implementation Guide (Next.js)](#implementation-guide)
4. [Astro Implementation](#astro-implementation)
5. [TanStack Start Implementation](#tanstack-start-implementation)
6. [Framework Comparison](#framework-comparison)
7. [Alternative Patterns](#alternative-patterns)
8. [Kubernetes Configuration](#kubernetes-configuration)
9. [When to Use What](#when-to-use-what)

---

## Architecture Overview

### Your Stack
```
┌─────────────────────────────────────────────────────────────────┐
│                        KUBERNETES CLUSTER                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │   Frontend   │     │  Fullstack   │     │    Rust      │    │
│  │   (React)    │     │  (Next.js)   │     │  API Gateway │    │
│  │              │     │              │     │   (tonic)    │    │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘    │
│         │                    │                    │             │
│         │    ┌───────────────┴───────────────┐    │             │
│         │    │                               │    │             │
│         ▼    ▼                               ▼    ▼             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Internal gRPC Services (Rust)               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │   │
│  │  │  User   │  │  Order  │  │ Payment │  │ Inventory│    │   │
│  │  │ Service │  │ Service │  │ Service │  │ Service │    │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Three Main Patterns

| Pattern | When to Use | Complexity |
|---------|-------------|------------|
| **BFF (Server Actions)** | Fullstack apps (Next.js, Nuxt) | Low |
| **gRPC-Web via Envoy** | Pure SPAs needing streaming | Medium |
| **HTTP via API Gateway** | External APIs, mobile, 3rd parties | Low |

---

## Recommended Pattern: BFF with Server Actions

**Best for: Next.js, Nuxt, SvelteKit, Remix, Astro, TanStack Start teams**

This is the **recommended approach** for your scenario because:
- Frontend devs work with familiar TypeScript/Node.js
- Type safety from proto → TypeScript generation
- No CORS issues (server-to-server calls)
- Can aggregate multiple gRPC calls
- Server-side rendering works seamlessly
- gRPC calls stay internal to the cluster

### Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                         BROWSER                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    React Components                       │   │
│  │   const user = await getUser(id)  // Server Action       │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP (automatic, handled by framework)
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    KUBERNETES CLUSTER                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Next.js Server (Node.js Pod)                │   │
│  │  ┌─────────────────────────────────────────────────┐    │   │
│  │  │           Server Actions / API Routes            │    │   │
│  │  │   'use server'                                   │    │   │
│  │  │   async function getUser(id) {                   │    │   │
│  │  │     return grpcClient.getUser({ id })            │    │   │
│  │  │   }                                              │    │   │
│  │  └──────────────────────┬──────────────────────────┘    │   │
│  │                         │                                │   │
│  │                         │ gRPC (HTTP/2, internal)       │   │
│  │                         ▼                                │   │
│  │  ┌─────────────────────────────────────────────────┐    │   │
│  │  │         ConnectRPC / gRPC-js Client             │    │   │
│  │  └──────────────────────┬──────────────────────────┘    │   │
│  └─────────────────────────┼────────────────────────────────┘   │
│                            │                                     │
│                            │ gRPC (cluster internal)            │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Rust gRPC Services (tonic)                  │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Why This Works Best

1. **Frontend devs only write TypeScript** - No Rust knowledge needed
2. **Type safety end-to-end** - Proto generates types for both sides
3. **Simple networking** - All gRPC is internal, HTTP to browser
4. **Aggregation** - One server action can call multiple services
5. **Caching** - Next.js caching works naturally
6. **Auth flows** - Handle auth in Node.js, pass to gRPC via metadata

---

## Implementation Guide

### Step 1: Proto Definition (Shared)

```protobuf
// proto/user/v1/user.proto
syntax = "proto3";

package user.v1;

service UserService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  rpc CreateUser(CreateUserRequest) returns (CreateUserResponse);
}

message User {
  string id = 1;
  string name = 2;
  string email = 3;
  int64 created_at = 4;
}

message GetUserRequest {
  string id = 1;
}

message GetUserResponse {
  User user = 1;
}

message ListUsersRequest {
  int32 page_size = 1;
  string page_token = 2;
}

message ListUsersResponse {
  repeated User users = 1;
  string next_page_token = 2;
}

message CreateUserRequest {
  string name = 1;
  string email = 2;
}

message CreateUserResponse {
  User user = 1;
}
```

### Step 2: Code Generation Setup

```yaml
# buf.gen.yaml - For TypeScript (Next.js)
version: v1
plugins:
  # Generate TypeScript message types
  - plugin: buf.build/bufbuild/es
    out: src/gen
    opt: target=ts

  # Generate ConnectRPC client
  - plugin: buf.build/connectrpc/es
    out: src/gen
    opt: target=ts
```

```bash
# Generate TypeScript types
npx buf generate proto
```

### Step 3: gRPC Client Setup (Node.js)

```typescript
// lib/grpc/client.ts
import { createClient } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import { UserService } from '@/gen/user/v1/user_connect';

// Environment-based service URLs (K8s DNS)
const GRPC_SERVICES = {
  user: process.env.USER_SERVICE_URL || 'http://user-service:50051',
  order: process.env.ORDER_SERVICE_URL || 'http://order-service:50051',
  // Add more services as needed
} as const;

// Create transport with sensible defaults
function createTransport(baseUrl: string) {
  return createGrpcTransport({
    baseUrl,
    httpVersion: '2',
    // Interceptors for logging, auth, etc.
    interceptors: [
      loggingInterceptor,
      authInterceptor,
      timeoutInterceptor,
    ],
  });
}

// Singleton clients (connection pooling)
let userClient: ReturnType<typeof createClient<typeof UserService>> | null = null;

export function getUserClient() {
  if (!userClient) {
    userClient = createClient(UserService, createTransport(GRPC_SERVICES.user));
  }
  return userClient;
}

// Interceptors
import { Interceptor } from '@connectrpc/connect';

const loggingInterceptor: Interceptor = (next) => async (req) => {
  const start = Date.now();
  console.log(`[gRPC] --> ${req.method.name}`);

  try {
    const res = await next(req);
    console.log(`[gRPC] <-- ${req.method.name} (${Date.now() - start}ms)`);
    return res;
  } catch (error) {
    console.error(`[gRPC] <-- ${req.method.name} ERROR (${Date.now() - start}ms)`, error);
    throw error;
  }
};

const authInterceptor: Interceptor = (next) => async (req) => {
  // Add auth token from request context if available
  // This would come from the Next.js request
  const token = getAuthToken(); // Implement based on your auth setup

  if (token) {
    req.header.set('authorization', `Bearer ${token}`);
  }

  return next(req);
};

const timeoutInterceptor: Interceptor = (next) => async (req) => {
  if (!req.timeoutMs) {
    req.timeoutMs = 10000; // 10 second default
  }
  return next(req);
};

function getAuthToken(): string | null {
  // Implement based on your auth setup
  // Could read from cookies, headers, etc.
  return null;
}
```

### Step 4: Server Actions (Next.js App Router)

```typescript
// app/actions/user.ts
'use server';

import { getUserClient } from '@/lib/grpc/client';
import { revalidatePath } from 'next/cache';
import { Code, ConnectError } from '@connectrpc/connect';

// Type-safe response wrapper
type ActionResult<T> =
  | { success: true; data: T }
  | { success: false; error: string; code?: string };

// Get single user
export async function getUser(id: string): Promise<ActionResult<User>> {
  try {
    const client = getUserClient();
    const response = await client.getUser({ id });

    if (!response.user) {
      return { success: false, error: 'User not found', code: 'NOT_FOUND' };
    }

    return {
      success: true,
      data: {
        id: response.user.id,
        name: response.user.name,
        email: response.user.email,
        createdAt: Number(response.user.createdAt),
      }
    };
  } catch (error) {
    return handleGrpcError(error);
  }
}

// List users with pagination
export async function listUsers(
  pageSize: number = 10,
  pageToken?: string
): Promise<ActionResult<{ users: User[]; nextPageToken: string }>> {
  try {
    const client = getUserClient();
    const response = await client.listUsers({ pageSize, pageToken: pageToken || '' });

    return {
      success: true,
      data: {
        users: response.users.map(u => ({
          id: u.id,
          name: u.name,
          email: u.email,
          createdAt: Number(u.createdAt),
        })),
        nextPageToken: response.nextPageToken,
      }
    };
  } catch (error) {
    return handleGrpcError(error);
  }
}

// Create user (with revalidation)
export async function createUser(
  name: string,
  email: string
): Promise<ActionResult<User>> {
  try {
    const client = getUserClient();
    const response = await client.createUser({ name, email });

    if (!response.user) {
      return { success: false, error: 'Failed to create user' };
    }

    // Revalidate the users list
    revalidatePath('/users');

    return {
      success: true,
      data: {
        id: response.user.id,
        name: response.user.name,
        email: response.user.email,
        createdAt: Number(response.user.createdAt),
      }
    };
  } catch (error) {
    return handleGrpcError(error);
  }
}

// Error handling helper
function handleGrpcError<T>(error: unknown): ActionResult<T> {
  if (error instanceof ConnectError) {
    // Map gRPC codes to user-friendly messages
    const errorMap: Record<number, string> = {
      [Code.NotFound]: 'Resource not found',
      [Code.AlreadyExists]: 'Resource already exists',
      [Code.InvalidArgument]: 'Invalid input provided',
      [Code.PermissionDenied]: 'Permission denied',
      [Code.Unauthenticated]: 'Please log in to continue',
      [Code.ResourceExhausted]: 'Rate limit exceeded, please try again',
      [Code.Unavailable]: 'Service temporarily unavailable',
    };

    return {
      success: false,
      error: errorMap[error.code] || 'An unexpected error occurred',
      code: Code[error.code],
    };
  }

  console.error('Unexpected error:', error);
  return { success: false, error: 'An unexpected error occurred' };
}

// TypeScript interface for frontend
interface User {
  id: string;
  name: string;
  email: string;
  createdAt: number;
}
```

### Step 5: React Components

```tsx
// app/users/page.tsx
import { listUsers } from '@/app/actions/user';
import { UserList } from '@/components/UserList';

export default async function UsersPage() {
  const result = await listUsers(20);

  if (!result.success) {
    return <div className="error">{result.error}</div>;
  }

  return <UserList initialUsers={result.data.users} />;
}
```

```tsx
// components/UserList.tsx
'use client';

import { useState, useTransition } from 'react';
import { listUsers, createUser } from '@/app/actions/user';

interface User {
  id: string;
  name: string;
  email: string;
  createdAt: number;
}

interface UserListProps {
  initialUsers: User[];
}

export function UserList({ initialUsers }: UserListProps) {
  const [users, setUsers] = useState(initialUsers);
  const [isPending, startTransition] = useTransition();
  const [error, setError] = useState<string | null>(null);

  async function handleCreateUser(formData: FormData) {
    const name = formData.get('name') as string;
    const email = formData.get('email') as string;

    startTransition(async () => {
      const result = await createUser(name, email);

      if (result.success) {
        setUsers(prev => [...prev, result.data]);
        setError(null);
      } else {
        setError(result.error);
      }
    });
  }

  return (
    <div>
      <h1>Users</h1>

      {error && <div className="error">{error}</div>}

      <form action={handleCreateUser}>
        <input name="name" placeholder="Name" required />
        <input name="email" type="email" placeholder="Email" required />
        <button type="submit" disabled={isPending}>
          {isPending ? 'Creating...' : 'Create User'}
        </button>
      </form>

      <ul>
        {users.map(user => (
          <li key={user.id}>
            {user.name} ({user.email})
          </li>
        ))}
      </ul>
    </div>
  );
}
```

### Step 6: API Routes (for external/non-action use)

```typescript
// app/api/users/route.ts
import { NextRequest, NextResponse } from 'next/server';
import { getUserClient } from '@/lib/grpc/client';
import { Code, ConnectError } from '@connectrpc/connect';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const pageSize = parseInt(searchParams.get('pageSize') || '10');
  const pageToken = searchParams.get('pageToken') || '';

  try {
    const client = getUserClient();
    const response = await client.listUsers({ pageSize, pageToken });

    return NextResponse.json({
      users: response.users,
      nextPageToken: response.nextPageToken,
    });
  } catch (error) {
    if (error instanceof ConnectError) {
      return NextResponse.json(
        { error: error.message },
        { status: grpcCodeToHttp(error.code) }
      );
    }
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const client = getUserClient();
    const response = await client.createUser({
      name: body.name,
      email: body.email,
    });

    return NextResponse.json({ user: response.user }, { status: 201 });
  } catch (error) {
    if (error instanceof ConnectError) {
      return NextResponse.json(
        { error: error.message },
        { status: grpcCodeToHttp(error.code) }
      );
    }
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

function grpcCodeToHttp(code: Code): number {
  const mapping: Record<number, number> = {
    [Code.Ok]: 200,
    [Code.InvalidArgument]: 400,
    [Code.Unauthenticated]: 401,
    [Code.PermissionDenied]: 403,
    [Code.NotFound]: 404,
    [Code.AlreadyExists]: 409,
    [Code.ResourceExhausted]: 429,
    [Code.Internal]: 500,
    [Code.Unavailable]: 503,
  };
  return mapping[code] || 500;
}
```

---

## Astro Implementation

Astro 4+ supports server endpoints and Actions, making it excellent for the BFF pattern with gRPC.

### Astro Setup

```bash
npm create astro@latest
npx astro add node  # Add Node.js adapter for SSR
npm install @connectrpc/connect @connectrpc/connect-node
```

```typescript
// astro.config.mjs
import { defineConfig } from 'astro/config';
import node from '@astrojs/node';

export default defineConfig({
  output: 'server',  // or 'hybrid' for mixed static/dynamic
  adapter: node({
    mode: 'standalone'
  }),
});
```

### Astro Actions (Astro 4.15+)

```typescript
// src/actions/index.ts
import { defineAction } from 'astro:actions';
import { z } from 'astro:schema';
import { getUserClient } from '@/lib/grpc/client';
import { Code, ConnectError } from '@connectrpc/connect';

export const server = {
  // Get user action
  getUser: defineAction({
    input: z.object({
      id: z.string().min(1),
    }),
    handler: async ({ id }) => {
      try {
        const client = getUserClient();
        const response = await client.getUser({ id });

        if (!response.user) {
          throw new Error('User not found');
        }

        return {
          id: response.user.id,
          name: response.user.name,
          email: response.user.email,
          createdAt: Number(response.user.createdAt),
        };
      } catch (error) {
        if (error instanceof ConnectError) {
          throw new Error(mapGrpcError(error.code));
        }
        throw error;
      }
    },
  }),

  // List users action
  listUsers: defineAction({
    input: z.object({
      pageSize: z.number().min(1).max(100).default(10),
      pageToken: z.string().optional(),
    }),
    handler: async ({ pageSize, pageToken }) => {
      const client = getUserClient();
      const response = await client.listUsers({
        pageSize,
        pageToken: pageToken || '',
      });

      return {
        users: response.users.map(u => ({
          id: u.id,
          name: u.name,
          email: u.email,
          createdAt: Number(u.createdAt),
        })),
        nextPageToken: response.nextPageToken,
      };
    },
  }),

  // Create user action
  createUser: defineAction({
    input: z.object({
      name: z.string().min(1).max(100),
      email: z.string().email(),
    }),
    handler: async ({ name, email }) => {
      const client = getUserClient();
      const response = await client.createUser({ name, email });

      return {
        id: response.user!.id,
        name: response.user!.name,
        email: response.user!.email,
        createdAt: Number(response.user!.createdAt),
      };
    },
  }),
};

function mapGrpcError(code: Code): string {
  const map: Record<number, string> = {
    [Code.NotFound]: 'Resource not found',
    [Code.AlreadyExists]: 'Resource already exists',
    [Code.InvalidArgument]: 'Invalid input',
    [Code.PermissionDenied]: 'Permission denied',
    [Code.Unauthenticated]: 'Please log in',
  };
  return map[code] || 'An error occurred';
}
```

### Astro Pages with Actions

```astro
---
// src/pages/users/index.astro
import { actions } from 'astro:actions';
import UserList from '@/components/UserList';

// Server-side data fetching via gRPC
const result = await Astro.callAction(actions.listUsers, { pageSize: 20 });

if (result.error) {
  return Astro.redirect('/error');
}

const { users, nextPageToken } = result.data;
---

<html>
  <head>
    <title>Users</title>
  </head>
  <body>
    <h1>Users</h1>
    <UserList client:load users={users} nextPageToken={nextPageToken} />
  </body>
</html>
```

### Astro React Component with Actions

```tsx
// src/components/UserList.tsx
import { actions } from 'astro:actions';
import { useState } from 'react';

interface User {
  id: string;
  name: string;
  email: string;
  createdAt: number;
}

interface Props {
  users: User[];
  nextPageToken: string;
}

export default function UserList({ users: initialUsers, nextPageToken: initialToken }: Props) {
  const [users, setUsers] = useState(initialUsers);
  const [pageToken, setPageToken] = useState(initialToken);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function loadMore() {
    if (!pageToken) return;
    setLoading(true);

    const result = await actions.listUsers({ pageSize: 20, pageToken });

    if (result.error) {
      setError(result.error.message);
    } else {
      setUsers(prev => [...prev, ...result.data.users]);
      setPageToken(result.data.nextPageToken);
    }
    setLoading(false);
  }

  async function handleCreate(formData: FormData) {
    const name = formData.get('name') as string;
    const email = formData.get('email') as string;

    const result = await actions.createUser({ name, email });

    if (result.error) {
      setError(result.error.message);
    } else {
      setUsers(prev => [...prev, result.data]);
      setError(null);
    }
  }

  return (
    <div>
      {error && <div className="error">{error}</div>}

      <form action={handleCreate}>
        <input name="name" placeholder="Name" required />
        <input name="email" type="email" placeholder="Email" required />
        <button type="submit">Create User</button>
      </form>

      <ul>
        {users.map(user => (
          <li key={user.id}>{user.name} ({user.email})</li>
        ))}
      </ul>

      {pageToken && (
        <button onClick={loadMore} disabled={loading}>
          {loading ? 'Loading...' : 'Load More'}
        </button>
      )}
    </div>
  );
}
```

### Astro API Endpoints (Alternative)

```typescript
// src/pages/api/users/[id].ts
import type { APIRoute } from 'astro';
import { getUserClient } from '@/lib/grpc/client';
import { Code, ConnectError } from '@connectrpc/connect';

export const GET: APIRoute = async ({ params }) => {
  const { id } = params;

  if (!id) {
    return new Response(JSON.stringify({ error: 'ID required' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  try {
    const client = getUserClient();
    const response = await client.getUser({ id });

    return new Response(JSON.stringify({ user: response.user }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (error) {
    if (error instanceof ConnectError) {
      const status = error.code === Code.NotFound ? 404 : 500;
      return new Response(JSON.stringify({ error: error.message }), {
        status,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    throw error;
  }
};
```

---

## TanStack Start Implementation

TanStack Start uses TanStack Router with server functions, providing excellent type safety.

### TanStack Start Setup

```bash
npm create @tanstack/start@latest
npm install @connectrpc/connect @connectrpc/connect-node
```

### Server Functions

```typescript
// app/server/grpc.ts
import { createServerFn } from '@tanstack/start';
import { getUserClient } from '@/lib/grpc/client';
import { Code, ConnectError } from '@connectrpc/connect';

// Get user server function
export const getUser = createServerFn('GET', async (id: string) => {
  try {
    const client = getUserClient();
    const response = await client.getUser({ id });

    if (!response.user) {
      throw new Error('User not found');
    }

    return {
      id: response.user.id,
      name: response.user.name,
      email: response.user.email,
      createdAt: Number(response.user.createdAt),
    };
  } catch (error) {
    if (error instanceof ConnectError) {
      throw new Error(mapGrpcError(error.code));
    }
    throw error;
  }
});

// List users server function
export const listUsers = createServerFn(
  'GET',
  async (params: { pageSize?: number; pageToken?: string }) => {
    const client = getUserClient();
    const response = await client.listUsers({
      pageSize: params.pageSize || 10,
      pageToken: params.pageToken || '',
    });

    return {
      users: response.users.map(u => ({
        id: u.id,
        name: u.name,
        email: u.email,
        createdAt: Number(u.createdAt),
      })),
      nextPageToken: response.nextPageToken,
    };
  }
);

// Create user server function
export const createUser = createServerFn(
  'POST',
  async (data: { name: string; email: string }) => {
    const client = getUserClient();
    const response = await client.createUser(data);

    return {
      id: response.user!.id,
      name: response.user!.name,
      email: response.user!.email,
      createdAt: Number(response.user!.createdAt),
    };
  }
);

function mapGrpcError(code: Code): string {
  const map: Record<number, string> = {
    [Code.NotFound]: 'Resource not found',
    [Code.AlreadyExists]: 'Resource already exists',
    [Code.InvalidArgument]: 'Invalid input',
    [Code.PermissionDenied]: 'Permission denied',
    [Code.Unauthenticated]: 'Please log in',
  };
  return map[code] || 'An error occurred';
}
```

### Route with Loader

```typescript
// app/routes/users.tsx
import { createFileRoute } from '@tanstack/react-router';
import { listUsers } from '@/server/grpc';

export const Route = createFileRoute('/users')({
  // Load data on server via gRPC
  loader: async () => {
    return listUsers({ pageSize: 20 });
  },
  component: UsersPage,
});

function UsersPage() {
  const { users, nextPageToken } = Route.useLoaderData();

  return (
    <div>
      <h1>Users</h1>
      <UserList users={users} nextPageToken={nextPageToken} />
    </div>
  );
}
```

### Route with Params

```typescript
// app/routes/users/$userId.tsx
import { createFileRoute } from '@tanstack/react-router';
import { getUser } from '@/server/grpc';

export const Route = createFileRoute('/users/$userId')({
  loader: async ({ params }) => {
    return getUser(params.userId);
  },
  component: UserDetailPage,
  errorComponent: ({ error }) => (
    <div className="error">
      <h1>Error</h1>
      <p>{error.message}</p>
    </div>
  ),
});

function UserDetailPage() {
  const user = Route.useLoaderData();

  return (
    <div>
      <h1>{user.name}</h1>
      <p>Email: {user.email}</p>
      <p>Created: {new Date(user.createdAt).toLocaleDateString()}</p>
    </div>
  );
}
```

### Component with Mutations

```tsx
// app/components/CreateUserForm.tsx
import { useMutation } from '@tanstack/react-query';
import { useRouter } from '@tanstack/react-router';
import { createUser } from '@/server/grpc';
import { useState } from 'react';

export function CreateUserForm() {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);

  const mutation = useMutation({
    mutationFn: createUser,
    onSuccess: () => {
      // Invalidate and refetch users
      router.invalidate();
      setError(null);
    },
    onError: (err) => {
      setError(err.message);
    },
  });

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);

    mutation.mutate({
      name: formData.get('name') as string,
      email: formData.get('email') as string,
    });
  }

  return (
    <form onSubmit={handleSubmit}>
      {error && <div className="error">{error}</div>}

      <input name="name" placeholder="Name" required />
      <input name="email" type="email" placeholder="Email" required />

      <button type="submit" disabled={mutation.isPending}>
        {mutation.isPending ? 'Creating...' : 'Create User'}
      </button>
    </form>
  );
}
```

### TanStack Query Integration

```typescript
// app/lib/queries.ts
import { queryOptions } from '@tanstack/react-query';
import { getUser, listUsers } from '@/server/grpc';

export const userQueries = {
  all: () => ['users'] as const,
  lists: () => [...userQueries.all(), 'list'] as const,
  list: (params: { pageSize?: number; pageToken?: string }) =>
    queryOptions({
      queryKey: [...userQueries.lists(), params],
      queryFn: () => listUsers(params),
    }),
  details: () => [...userQueries.all(), 'detail'] as const,
  detail: (id: string) =>
    queryOptions({
      queryKey: [...userQueries.details(), id],
      queryFn: () => getUser(id),
    }),
};

// Usage in component
import { useSuspenseQuery } from '@tanstack/react-query';

function UserDetail({ id }: { id: string }) {
  const { data: user } = useSuspenseQuery(userQueries.detail(id));

  return <div>{user.name}</div>;
}
```

---

## Framework Comparison

| Feature | Next.js | Astro | TanStack Start |
|---------|---------|-------|----------------|
| **Server Functions** | Server Actions | Actions + API Routes | Server Functions |
| **Data Fetching** | RSC, Server Actions | Astro.callAction, loader | Loader, Server Fn |
| **Rendering** | SSR, SSG, ISR | SSR, SSG, Hybrid | SSR |
| **Routing** | File-based | File-based | Type-safe (TanStack Router) |
| **Bundle Size** | Medium | Small (islands) | Small-Medium |
| **React Support** | Native | Via integration | Native |
| **Type Safety** | Good | Good | Excellent (router) |
| **Best For** | Full apps | Content + islands | Type-safe SPA/MPA |

### When to Choose

| Framework | Use When |
|-----------|----------|
| **Next.js** | Large apps, team familiarity, Vercel deployment |
| **Astro** | Content-heavy sites, need islands architecture, performance focus |
| **TanStack Start** | Type-safety priority, complex routing, TanStack Query users |

---

## Alternative Patterns

### Pattern 2: gRPC-Web via Envoy (for SPAs)

Use when: Pure SPA, need streaming, no Node.js server

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     Browser     │────▶│  Envoy Proxy    │────▶│  Rust Services  │
│   (gRPC-Web)    │     │ (gRPC-Web→gRPC) │     │    (tonic)      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

```yaml
# envoy.yaml
static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address:
          address: 0.0.0.0
          port_value: 8080
      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                codec_type: auto
                stat_prefix: ingress_http
                route_config:
                  name: local_route
                  virtual_hosts:
                    - name: local_service
                      domains: ["*"]
                      routes:
                        - match:
                            prefix: "/user.v1.UserService"
                          route:
                            cluster: user_service
                      cors:
                        allow_origin_string_match:
                          - prefix: "*"
                        allow_methods: GET, PUT, DELETE, POST, OPTIONS
                        allow_headers: keep-alive,user-agent,cache-control,content-type,content-transfer-encoding,x-accept-content-transfer-encoding,x-accept-response-streaming,x-user-agent,x-grpc-web,grpc-timeout,authorization
                        expose_headers: grpc-status,grpc-message
                http_filters:
                  - name: envoy.filters.http.grpc_web
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.grpc_web.v3.GrpcWeb
                  - name: envoy.filters.http.cors
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.cors.v3.Cors
                  - name: envoy.filters.http.router
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
  clusters:
    - name: user_service
      connect_timeout: 0.25s
      type: logical_dns
      lb_policy: round_robin
      http2_protocol_options: {}
      load_assignment:
        cluster_name: user_service
        endpoints:
          - lb_endpoints:
              - endpoint:
                  address:
                    socket_address:
                      address: user-service
                      port_value: 50051
```

```typescript
// Browser client with gRPC-Web
import { createClient } from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { UserService } from '@/gen/user/v1/user_connect';

const transport = createGrpcWebTransport({
  baseUrl: 'https://api.example.com', // Envoy proxy
});

export const userClient = createClient(UserService, transport);
```

### Pattern 3: HTTP via Rust API Gateway

Use when: External APIs, mobile apps, third-party integrations

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     Client      │────▶│  Rust Gateway   │────▶│  Rust Services  │
│  (HTTP/REST)    │     │  (axum + tonic) │     │    (tonic)      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

```rust
// Rust API Gateway (simplified)
use axum::{routing::get, Router, Json, extract::Path};
use tonic::transport::Channel;

// gRPC client
lazy_static! {
    static ref USER_CLIENT: UserServiceClient<Channel> = {
        let channel = Channel::from_static("http://user-service:50051")
            .connect_lazy();
        UserServiceClient::new(channel)
    };
}

// HTTP handler that calls gRPC
async fn get_user(Path(id): Path<String>) -> Result<Json<UserResponse>, AppError> {
    let request = tonic::Request::new(GetUserRequest { id });
    let response = USER_CLIENT.clone().get_user(request).await?;

    Ok(Json(UserResponse::from(response.into_inner())))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/v1/users/:id", get(get_user))
        .route("/api/v1/users", get(list_users).post(create_user));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

---

## Kubernetes Configuration

### Service Definitions

```yaml
# k8s/base/user-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: user-service
  labels:
    app: user-service
spec:
  ports:
    - name: grpc
      port: 50051
      targetPort: 50051
      protocol: TCP
  selector:
    app: user-service
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: user-service
  template:
    metadata:
      labels:
        app: user-service
    spec:
      containers:
        - name: user-service
          image: your-registry/user-service:latest
          ports:
            - containerPort: 50051
          env:
            - name: RUST_LOG
              value: "info"
          resources:
            requests:
              memory: "128Mi"
              cpu: "100m"
            limits:
              memory: "256Mi"
              cpu: "500m"
          readinessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 10
            periodSeconds: 20
```

```yaml
# k8s/base/nextjs-app.yaml
apiVersion: v1
kind: Service
metadata:
  name: nextjs-app
spec:
  ports:
    - name: http
      port: 3000
      targetPort: 3000
  selector:
    app: nextjs-app
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nextjs-app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nextjs-app
  template:
    metadata:
      labels:
        app: nextjs-app
    spec:
      containers:
        - name: nextjs-app
          image: your-registry/nextjs-app:latest
          ports:
            - containerPort: 3000
          env:
            # gRPC service URLs (K8s internal DNS)
            - name: USER_SERVICE_URL
              value: "http://user-service:50051"
            - name: ORDER_SERVICE_URL
              value: "http://order-service:50051"
            - name: NODE_ENV
              value: "production"
          resources:
            requests:
              memory: "256Mi"
              cpu: "200m"
            limits:
              memory: "512Mi"
              cpu: "1000m"
```

### Ingress Configuration

```yaml
# k8s/base/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: main-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - app.example.com
        - api.example.com
      secretName: tls-secret
  rules:
    # Frontend/Fullstack app
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: nextjs-app
                port:
                  number: 3000
    # REST API Gateway (for external clients)
    - host: api.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: rust-api-gateway
                port:
                  number: 8080
```

### Network Policy (Security)

```yaml
# k8s/base/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: grpc-internal-only
spec:
  podSelector:
    matchLabels:
      tier: grpc-backend
  policyTypes:
    - Ingress
  ingress:
    # Only allow traffic from internal services
    - from:
        - podSelector:
            matchLabels:
              tier: frontend
        - podSelector:
            matchLabels:
              tier: api-gateway
      ports:
        - protocol: TCP
          port: 50051
```

---

## When to Use What

### Decision Matrix

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           DECISION TREE                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Is this a fullstack app (Next.js, Nuxt, etc.)?                        │
│  │                                                                       │
│  ├── YES ──▶ Use Server Actions + gRPC Client (RECOMMENDED)            │
│  │           • Type-safe, simple, no CORS                               │
│  │           • gRPC calls stay internal                                 │
│  │                                                                       │
│  └── NO                                                                  │
│       │                                                                  │
│       ▼                                                                  │
│  Is this a pure SPA needing real-time/streaming?                        │
│  │                                                                       │
│  ├── YES ──▶ Use gRPC-Web via Envoy                                    │
│  │           • Streaming support                                        │
│  │           • Requires Envoy sidecar                                   │
│  │                                                                       │
│  └── NO                                                                  │
│       │                                                                  │
│       ▼                                                                  │
│  Is this for external clients (mobile, 3rd party)?                      │
│  │                                                                       │
│  ├── YES ──▶ Use HTTP REST via Rust API Gateway                        │
│  │           • Standard HTTP, easy integration                          │
│  │           • OpenAPI documentation                                    │
│  │                                                                       │
│  └── NO ──▶ Re-evaluate requirements                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Summary Table

| Scenario | Pattern | Proto Gen | Client Code |
|----------|---------|-----------|-------------|
| Next.js app | Server Actions | `@connectrpc/connect` | Node.js gRPC |
| Nuxt app | Server routes | `@connectrpc/connect` | Node.js gRPC |
| React SPA + streaming | gRPC-Web | `@connectrpc/connect-web` | Browser gRPC-Web |
| Mobile app | HTTP Gateway | N/A | REST client |
| External API | HTTP Gateway | N/A | REST client |
| Internal tool | Direct gRPC | Language-specific | Native gRPC |

### Recommended Stack

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         RECOMMENDED ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  EXTERNAL                                                                │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Browser ──▶ Next.js (Server Actions) ──▶ gRPC Services         │   │
│  │  Mobile  ──▶ Rust HTTP Gateway ──────────▶ gRPC Services        │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  INTERNAL (K8s)                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Service A (Rust) ──gRPC──▶ Service B (Rust)                    │   │
│  │  Service C (Go)   ──gRPC──▶ Service B (Rust)                    │   │
│  │  CronJob (Node)   ──gRPC──▶ Service A (Rust)                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  TOOLS                                                                   │
│  • buf - Proto management & code generation                              │
│  • ConnectRPC - Type-safe clients (Node.js & Browser)                   │
│  • tonic - Rust gRPC server                                             │
│  • Envoy - gRPC-Web proxy (only if needed)                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start Checklist

### For Fullstack Teams (Next.js)

- [ ] Set up buf.yaml and buf.gen.yaml in shared proto repo
- [ ] Generate TypeScript types with `@connectrpc/connect`
- [ ] Create gRPC client singleton with interceptors
- [ ] Create Server Actions that wrap gRPC calls
- [ ] Add proper error handling and type mapping
- [ ] Configure K8s services with internal DNS
- [ ] Set up health checks for all services

### For Frontend Teams (Pure SPA)

- [ ] Deploy Envoy sidecar for gRPC-Web translation
- [ ] Generate browser-compatible client with `@connectrpc/connect-web`
- [ ] Configure CORS properly in Envoy
- [ ] Handle streaming if needed
- [ ] Set up proper error boundaries

### For Backend/Platform Teams

- [ ] Ensure all Rust services implement health checks
- [ ] Set up proper K8s network policies
- [ ] Configure resource limits and HPA
- [ ] Set up monitoring (Prometheus metrics)
- [ ] Document proto files and breaking change policy
