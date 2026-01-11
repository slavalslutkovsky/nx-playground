#!/usr/bin/env nu

# Monorepo Nu Scripts - Main Entry Point
# Usage: nu scripts/nu/mod.nu <command> [args]

export use common.nu *

const SCRIPT_DIR = path self

# ============================================================================
# Top-level up/down commands - Full environment lifecycle
# ============================================================================

# Bring up the full development environment (cluster + services)
export def "main up" [
    --cloud (-c): string = "local"   # Cloud provider: local, aws, gcp, azure
    --name (-n): string = "dev"      # Cluster name
    --workers (-w): int = 1          # Number of worker nodes
    --k8s-version: string = ""       # Kubernetes version (empty = latest)
    --ingress (-i)                   # Enable ingress (ports 80, 443)
    --ha                             # Enable HA mode (multiple workers)
    --skip-dbs                       # Skip database deployment
    --skip-obs                       # Skip observability stack
    --dry-run                        # Preview without executing
    --verbose (-v)                   # Verbose output
] {
    let workers = if $ha { ([$workers 2] | math max) } else { $workers }

    validate-provider $cloud

    if $verbose {
        info $"Starting environment: cloud=($cloud) name=($name) workers=($workers)"
    }

    match $cloud {
        "local" => {
            require-bin "kind"
            require-bin "kubectl"

            if $dry_run {
                info "[DRY-RUN] Would create Kind cluster and deploy services"
                return
            }

            # Create Kind cluster
            info $"Creating Kind cluster: ($name)"
            ^nu ($SCRIPT_DIR | path dirname | path join "cluster.nu") create -n $name -w $workers --ingress

            # Create app namespaces
            info "Creating app namespaces..."
            create-app-namespaces

            # Deploy databases
            if not $skip_dbs {
                info "Deploying database services..."
                ^nu ($SCRIPT_DIR | path dirname | path join "cluster.nu") setup --dbs
            }

            # Deploy observability
            if not $skip_obs {
                info "Deploying observability stack..."
                do { ^nu ($SCRIPT_DIR | path dirname | path join "cluster.nu") observability --target dev } | complete
            }

            success $"Environment '($name)' is up!"
            print ""
            print "Next steps:"
            print "  - Run 'tilt up' to start application development"
            print "  - Run 'just down' to tear down the environment"
        }
        "aws" => {
            require-bin "aws"
            require-bin "eksctl"

            if $dry_run {
                info "[DRY-RUN] Would create EKS cluster"
                return
            }

            info "Creating AWS EKS cluster..."
            # TODO: Implement AWS cluster creation
            warn "AWS cluster creation not yet implemented"
        }
        "gcp" => {
            require-bin "gcloud"

            if $dry_run {
                info "[DRY-RUN] Would create GKE cluster"
                return
            }

            info "Creating GCP GKE cluster..."
            # TODO: Implement GCP cluster creation
            warn "GCP cluster creation not yet implemented"
        }
        "azure" => {
            require-bin "az"

            if $dry_run {
                info "[DRY-RUN] Would create AKS cluster"
                return
            }

            info "Creating Azure AKS cluster..."
            # TODO: Implement Azure cluster creation
            warn "Azure cluster creation not yet implemented"
        }
    }
}

# Tear down the development environment
export def "main down" [
    --cloud (-c): string = "local"   # Cloud provider: local, aws, gcp, azure
    --name (-n): string = "dev"      # Cluster name
    --keep-cluster                   # Keep cluster, only remove resources
    --verbose (-v)                   # Verbose output
] {
    validate-provider $cloud

    if $verbose {
        info $"Tearing down environment: cloud=($cloud) name=($name)"
    }

    match $cloud {
        "local" => {
            require-bin "kind"

            if $keep_cluster {
                info "Removing resources but keeping cluster..."
                do { kubectl delete ns dbs } | complete
                do { kubectl delete ns monitoring } | complete
                success "Resources removed, cluster kept"
            } else {
                info $"Deleting Kind cluster: ($name)"
                ^nu ($SCRIPT_DIR | path dirname | path join "cluster.nu") delete $name
                success $"Environment '($name)' is down"
            }
        }
        "aws" => {
            info "Deleting AWS EKS cluster..."
            warn "AWS cluster deletion not yet implemented"
        }
        "gcp" => {
            info "Deleting GCP GKE cluster..."
            warn "GCP cluster deletion not yet implemented"
        }
        "azure" => {
            info "Deleting Azure AKS cluster..."
            warn "Azure cluster deletion not yet implemented"
        }
    }
}

# Show environment status
export def "main status" [
    --cloud (-c): string = "local"   # Cloud provider
] {
    match $cloud {
        "local" => {
            ^nu ($SCRIPT_DIR | path dirname | path join "cluster.nu") list
            ^nu ($SCRIPT_DIR | path dirname | path join "cluster.nu") status
        }
        _ => {
            warn $"Status for ($cloud) not yet implemented"
        }
    }
}

# Validate cloud provider
def validate-provider [cloud: string] {
    let valid = ["local" "aws" "gcp" "azure"]
    if not ($cloud in $valid) {
        error $"Invalid cloud provider: ($cloud). Valid options: ($valid | str join ', ')"
        exit 1
    }
}

# Create app namespaces based on apps directory structure
def create-app-namespaces [] {
    let apps_dir = "apps"

    if not ($apps_dir | path exists) {
        warn "apps/ directory not found, skipping namespace creation"
        return
    }

    # Get top-level directories in apps/ - these become namespaces
    let namespaces = (ls $apps_dir | where type == dir | get name | path basename)

    for ns in $namespaces {
        info $"  Creating namespace: ($ns)"
        do { kubectl create namespace $ns } | complete
    }

    success $"Created ($namespaces | length) app namespaces: ($namespaces | str join ', ')"
}

# ============================================================================
# Subcommand delegators
# ============================================================================

# Setup commands - install dependencies, build, check
export def --wrapped "main setup" [...args] {
    let script = ($SCRIPT_DIR | path dirname | path join "setup.nu")
    ^nu $script ...$args
}

# Local development - docker compose, prune, kompose
export def --wrapped "main dev" [...args] {
    let script = ($SCRIPT_DIR | path dirname | path join "local-dev.nu")
    ^nu $script ...$args
}

# Cluster management - create, delete, status, gitops
export def --wrapped "main cluster" [...args] {
    let script = ($SCRIPT_DIR | path dirname | path join "cluster.nu")
    ^nu $script ...$args
}

# Secrets management - fetch, verify, load
export def --wrapped "main secrets" [...args] {
    let script = ($SCRIPT_DIR | path dirname | path join "secrets.nu")
    ^nu $script ...$args
}

# Main help
def main [] {
    print "Monorepo Nu Scripts"
    print ""
    print "Usage: nu scripts/nu/mod.nu <command> [args]"
    print ""
    print "Quick Start:"
    print "  up      - Bring up full dev environment (cluster + services)"
    print "  down    - Tear down dev environment"
    print "  status  - Show environment status"
    print ""
    print "Subcommands:"
    print "  setup   - Project setup (install, build, check, test)"
    print "  dev     - Docker compose (up, down, logs, kompose, prune)"
    print "  cluster - Kind cluster (create, delete, gitops, observability)"
    print "  secrets - Secrets management (fetch, verify, load)"
    print ""
    print "Examples:"
    print "  nu scripts/nu/mod.nu up                    # Local Kind cluster + services"
    print "  nu scripts/nu/mod.nu up -c aws -n prod     # AWS EKS cluster"
    print "  nu scripts/nu/mod.nu down                  # Tear down local env"
    print "  nu scripts/nu/mod.nu status                # Show cluster status"
    print ""
    print "  nu scripts/nu/mod.nu setup install --all"
    print "  nu scripts/nu/mod.nu dev up -d"
    print "  nu scripts/nu/mod.nu cluster create -n dev -w 2"
    print "  nu scripts/nu/mod.nu secrets fetch"
}
