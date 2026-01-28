# NxPlayground

<a alt="Nx logo" href="https://nx.dev" target="_blank" rel="noreferrer"><img src="https://raw.githubusercontent.com/nrwl/nx/master/images/nx-logo.png" width="45"></a>

✨ Your new, shiny [Nx workspace](https://nx.dev) is almost ready ✨.

Run `npx nx graph` to visually explore what got created. Now, let's get you up to speed!

## Finish your CI setup

[Click here to finish setting up your workspace!](https://cloud.nx.app/connect/4NCocrYDY9)


## Run tasks

To run tasks with Nx use:

```sh
npx nx <target> <project-name>
```

For example:

```sh
npx nx build myproject
```

These targets are either [inferred automatically](https://nx.dev/concepts/inferred-tasks?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) or defined in the `project.json` or `package.json` files.

[More about running tasks in the docs &raquo;](https://nx.dev/features/run-tasks?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

## Add new projects

While you could add new projects to your workspace manually, you might want to leverage [Nx plugins](https://nx.dev/concepts/nx-plugins?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) and their [code generation](https://nx.dev/features/generate-code?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) feature.

To install a new plugin you can use the `nx add` command. Here's an example of adding the React plugin:
```sh
npx nx add @nx/react
```

Use the plugin's generator to create new projects. For example, to create a new React app or library:

```sh
# Generate an app
npx nx g @nx/react:app demo

# Generate a library
npx nx g @nx/react:lib some-lib
```

You can use `npx nx list` to get a list of installed plugins. Then, run `npx nx list <plugin-name>` to learn about more specific capabilities of a particular plugin. Alternatively, [install Nx Console](https://nx.dev/getting-started/editor-setup?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) to browse plugins and generators in your IDE.

[Learn more about Nx plugins &raquo;](https://nx.dev/concepts/nx-plugins?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) | [Browse the plugin registry &raquo;](https://nx.dev/plugin-registry?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)


[Learn more about Nx on CI](https://nx.dev/ci/intro/ci-with-nx#ready-get-started-with-your-provider?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

## Install Nx Console

Nx Console is an editor extension that enriches your developer experience. It lets you run tasks, generate code, and improves code autocompletion in your IDE. It is available for VSCode and IntelliJ.

[Install Nx Console &raquo;](https://nx.dev/getting-started/editor-setup?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

## Local Development Environment

This project uses [Nushell](https://www.nushell.sh/) scripts and [KCL](https://kcl-lang.io/) for infrastructure automation.

### Quick Start

```bash
# Bring up full dev environment (Kind cluster + databases + observability)
just up

# Tear down environment
just down

# Show environment status
just status
```

### Environment Options

```bash
# Custom cluster name and workers
just up -n prod -w 3

# With HA mode (2+ workers)
just up --ha

# Skip optional components
just up --skip-dbs --skip-obs

# Dry run (preview without executing)
just up --dry-run

# Cloud providers (coming soon)
just up -c aws -n prod
just up -c gcp -n prod
just up -c azure -n prod
```

### What `just up` Does

1. Creates a Kind cluster with ingress support
2. Creates app namespaces (based on `apps/` directory structure)
3. Deploys databases via kompose (MongoDB, PostgreSQL, Redis, etc.)
4. Deploys observability stack (Prometheus, Grafana)

### Nu Scripts

The automation is powered by Nushell scripts in `scripts/nu/`:

```bash
# Direct nu script usage
nu scripts/nu/mod.nu up                    # Full environment
nu scripts/nu/mod.nu down                  # Tear down
nu scripts/nu/mod.nu status                # Show status

# Subcommands
nu scripts/nu/mod.nu setup install --all   # Install dependencies
nu scripts/nu/mod.nu dev up -d             # Docker compose only
nu scripts/nu/mod.nu cluster create -n dev # Cluster only
nu scripts/nu/mod.nu secrets fetch         # Fetch secrets from Vault
```

### KCL Configuration

Kind cluster configuration is defined in KCL (`scripts/kcl/cluster/`):

```bash
# Generate cluster config
kcl run scripts/kcl/cluster/main.k -D name=dev -D workers=2 -D ingress=true

# Run tests
kcl run scripts/kcl/cluster/kind_test.k
```

### Prerequisites

Install required tools:

```bash
# Via nu script
nu scripts/nu/mod.nu setup install --k8s

# Or manually (macOS)
brew install kind kubectl kustomize tilt helm flux kcl nushell
```

## Project Setup

### Environment Variables

This project uses `direnv` for automatic environment variable loading.

**Setup:**
```bash
# 1. Install direnv
brew install direnv  # macOS
# or
apt install direnv   # Ubuntu/Debian

# 2. Add to your shell (~/.zshrc or ~/.bashrc)
eval "$(direnv hook zsh)"  # or bash

# 3. Copy environment template
cp .env.example .env

# 4. Edit .env with your actual values
vim .env

# 5. Allow direnv
direnv allow
```

Now environment variables automatically load when you `cd` into the project!

### Database Migrations

This project uses [SeaORM](https://www.sea-ql.org/SeaORM/) with `sea-orm-cli` for database migrations.

**Quick Start:**
```bash
# Install sea-orm-cli
cargo install sea-orm-cli

# Run migrations (with direnv setup)
sea-orm-cli migrate up

# Or without direnv
DATABASE_URL=postgres://user:pass@localhost/db \
  sea-orm-cli migrate -d libs/migration up
```

**Common Commands:**
```bash
sea-orm-cli migrate up              # Run pending migrations
sea-orm-cli migrate down            # Rollback last migration
sea-orm-cli migrate status          # Check migration status
sea-orm-cli migrate fresh           # Drop all & re-run (dev only!)
```

**Create New Migration:**
```bash
cd libs/migration
sea-orm-cli migrate -d . generate <migration_name>
```

For complete documentation, see [libs/migration/README.md](libs/migration/README.md)

### Running the API

```bash
# Development (with auto-migration)
RUN_MIGRATIONS=true cargo run -p zerg_api

# Production (run migrations separately first)
sea-orm-cli migrate up
cargo run -p zerg_api
```

## Useful links

Learn more:

- [Learn about Nx on CI](https://nx.dev/ci/intro/ci-with-nx?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)
- [Releasing Packages with Nx release](https://nx.dev/features/manage-releases?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)
- [What are Nx plugins?](https://nx.dev/concepts/nx-plugins?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

And join the Nx community:
- [Discord](https://go.nx.dev/community)
- [Follow us on X](https://twitter.com/nxdevtools) or [LinkedIn](https://www.linkedin.com/company/nrwl)
- [Our Youtube channel](https://www.youtube.com/@nxdevtools)
- [Our blog](https://nx.dev/blog?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)
