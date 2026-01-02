# =============================================================================
# Tilt Configuration - Parallel Development Environment
# =============================================================================
# Usage:
#   tilt up                           # Default: k8s mode with apps
#   tilt up -- --mode=local           # Local mode (no k8s, just cargo/bun)
#   tilt up -- --mode=k8s             # K8s mode with apps only
#   tilt up -- --mode=full            # Full stack: apps + all databases
#   tilt up -- --databases=postgres,redis  # Selective databases
#   tilt up -- --apps=zerg-api,zerg-web    # Selective apps
#   tilt up -- --export               # Export k8s manifests only
# =============================================================================

# Configuration options
config.define_string('mode', args=True, usage='Development mode: local, k8s, full')
config.define_string_list('databases', args=True, usage='Databases to run: postgres,redis,mongo,influxdb2,qdrant,neo4j,arangodb,milvus')
config.define_string_list('apps', args=True, usage='Apps to run: zerg-api,zerg-tasks,zerg-web,zerg-mongo-api,inventory-api,products-api,products-nats-api,storefront')
config.define_bool('export', args=True, usage='Export manifests only (no deployment)')
config.define_bool('port-forwards', args=True, usage='Enable port forwards for external services')

cfg = config.parse()

# Defaults
mode = cfg.get('mode', 'k8s')
selected_databases = cfg.get('databases', [])
selected_apps = cfg.get('apps', [])
export_only = cfg.get('export', False)
enable_port_forwards = cfg.get('port-forwards', True)

# =============================================================================
# Export Mode - Generate manifests without running
# =============================================================================
if export_only:
    local('mkdir -p ./dist/k8s')
    local('kubectl kustomize ./k8s/databases > ./dist/k8s/databases.yaml')
    local('kubectl kustomize ./apps/zerg/api/k8s/kustomize/overlays/dev > ./dist/k8s/zerg-api.yaml')
    local('kubectl kustomize ./apps/zerg/tasks/k8s/kustomize/overlays/dev > ./dist/k8s/zerg-tasks.yaml')
    local('kubectl kustomize ./apps/zerg/web/k8s/kustomize/overlays/dev > ./dist/k8s/zerg-web.yaml')
    local('kubectl kustomize ./apps/zerg-mongo-api/k8s/kustomize/overlays/dev > ./dist/k8s/zerg-mongo-api.yaml')
    local('kubectl kustomize ./apps/inventory-api/k8s/kustomize/overlays/dev > ./dist/k8s/inventory-api.yaml')
    local('kubectl kustomize ./apps/products-api/k8s/kustomize/overlays/dev > ./dist/k8s/products-api.yaml')
    local('kubectl kustomize ./apps/products-nats-api/k8s/kustomize/overlays/dev > ./dist/k8s/products-nats-api.yaml')
    local('kubectl kustomize ./apps/storefront/k8s/kustomize/overlays/dev > ./dist/k8s/storefront.yaml')
    print('Manifests exported to ./dist/k8s/')
    # Exit early - don't start any resources
    config.clear_enabled_resources()

# =============================================================================
# Database Definitions
# =============================================================================
all_databases = ['postgres', 'redis', 'mongo', 'influxdb2', 'qdrant', 'neo4j', 'arangodb', 'milvus', 'nats']

# If no specific databases selected and mode is 'full', use core databases
if mode == 'full' and not selected_databases:
    selected_databases = ['postgres', 'redis', 'mongo', 'influxdb2']

# =============================================================================
# LOCAL MODE - Run services directly with cargo/bun (no Docker/K8s)
# =============================================================================
if mode == 'local':
    # Parallel local Rust services - Zerg ecosystem
    local_resource(
        'local-zerg-tasks',
        serve_cmd='cargo run -p zerg_tasks --color always',
        deps=['apps/zerg/tasks/src', 'libs'],
        labels=['local-backend'],
        allow_parallel=True,
    )

    local_resource(
        'local-zerg-api',
        serve_cmd='cargo run -p zerg_api --color always',
        deps=['apps/zerg/api/src', 'libs'],
        labels=['local-backend'],
        resource_deps=['local-zerg-tasks'],
        allow_parallel=True,
    )

    local_resource(
        'local-zerg-mongo-api',
        serve_cmd='cargo run -p zerg_mongo_api --color always',
        deps=['apps/zerg-mongo-api/src', 'libs'],
        labels=['local-backend'],
        allow_parallel=True,
    )

    local_resource(
        'local-zerg-web',
        serve_cmd='bun nx run zerg-web:dev',
        dir='.',
        deps=['apps/zerg/web/src'],
        labels=['local-frontend'],
        allow_parallel=True,
    )

    # Inventory API (gRPC + Dapr)
    local_resource(
        'local-inventory-api',
        serve_cmd='cargo run -p inventory_api --color always',
        deps=['apps/inventory-api/src', 'libs'],
        labels=['local-backend'],
        allow_parallel=True,
    )

    # Products ecosystem
    local_resource(
        'local-products-api',
        serve_cmd='cargo run -p products_api --color always',
        deps=['apps/products-api/src', 'libs'],
        labels=['local-backend'],
        allow_parallel=True,
    )

    local_resource(
        'local-products-nats-api',
        serve_cmd='cd apps/products-nats-api && bun run start:dev',
        deps=['apps/products-nats-api/src'],
        labels=['local-backend'],
        allow_parallel=True,
    )

    # Storefront (Astro SSR)
    local_resource(
        'local-storefront',
        serve_cmd='cd apps/storefront && bun run dev',
        deps=['apps/storefront/src'],
        labels=['local-frontend'],
        allow_parallel=True,
    )

    # Databases via docker-compose (parallel)
    local_resource(
        'local-databases',
        serve_cmd='docker compose -f manifests/dockers/compose.yaml up postgres redis mongo influxdb2 nats',
        labels=['local-databases'],
        allow_parallel=True,
    )

# =============================================================================
# K8S MODE - Deploy to Kubernetes
# =============================================================================
if mode in ['k8s', 'full']:
    # Include app Tiltfiles - Zerg ecosystem
    if not selected_apps or 'zerg-api' in selected_apps:
        include('./apps/zerg/api/Tiltfile')
    if not selected_apps or 'zerg-tasks' in selected_apps:
        include('./apps/zerg/tasks/Tiltfile')
    if not selected_apps or 'zerg-web' in selected_apps:
        include('./apps/zerg/web/Tiltfile')
    if not selected_apps or 'zerg-mongo-api' in selected_apps:
        include('./apps/zerg-mongo-api/Tiltfile')

    # Inventory service (gRPC + Dapr)
    if not selected_apps or 'inventory-api' in selected_apps:
        include('./apps/inventory-api/Tiltfile')

    # Products ecosystem
    if not selected_apps or 'products-api' in selected_apps:
        include('./apps/products-api/Tiltfile')
    if not selected_apps or 'products-nats-api' in selected_apps:
        include('./apps/products-nats-api/Tiltfile')

    # Frontend
    if not selected_apps or 'storefront' in selected_apps:
        include('./apps/storefront/Tiltfile')

# =============================================================================
# Database Installation (K8s)
# =============================================================================
def install_database(name, namespace='dbs'):
    """Install a database from k8s/databases via kustomize"""
    k8s_yaml(kustomize('./k8s/databases'),
             allow_duplicates=True)

# Install selected databases
if mode == 'full' or selected_databases:
    # Apply database kustomization
    k8s_yaml(kustomize('./k8s/databases'))

    # Create port forwards for each database
    db_ports = {
        'postgres': ('5433', '5432'),
        'redis': ('6379', '6379'),
        'mongo': ('27017', '27017'),
        'influxdb2': ('8086', '8086'),
        'qdrant': ('6333', '6333'),
        'neo4j': ('7474', '7474'),
        'arangodb': ('8529', '8529'),
        'milvus': ('19530', '19530'),
        'nats': ('4222', '4222'),
    }

    for db in (selected_databases if selected_databases else all_databases):
        if db in db_ports:
            local_port, container_port = db_ports[db]
            k8s_resource(
                db,
                labels=['databases'],
            )

# =============================================================================
# Port Forwards for External Services (always available)
# =============================================================================
if enable_port_forwards and mode in ['k8s', 'full']:
    local_resource(
        'redis-port-forward',
        serve_cmd='kubectl port-forward -n dbs deployment/redis 6379:6379',
        labels=['port-forwards'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 6379'])
        )
    )

    local_resource(
        'postgres-port-forward',
        serve_cmd='kubectl port-forward -n dbs deployment/postgres 5433:5432',
        labels=['port-forwards'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 5433'])
        )
    )

    local_resource(
        'mongo-port-forward',
        serve_cmd='kubectl port-forward -n dbs deployment/mongo 27017:27017',
        labels=['port-forwards'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 27017'])
        )
    )

    local_resource(
        'influxdb2-port-forward',
        serve_cmd='kubectl port-forward -n dbs deployment/influxdb2 8086:8086',
        labels=['port-forwards'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 8086'])
        )
    )

    local_resource(
        'komoplane-port-forward',
        serve_cmd='kubectl port-forward -n crossplane-system deployment/komoplane 8090:8090',
        labels=['platform'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 8090'])
        )
    )

    local_resource(
        'grafana-port-forward',
        serve_cmd='kubectl port-forward -n monitoring deployment/monitoring-grafana 3000:3000',
        labels=['monitoring'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 3000'])
        )
    )

    local_resource(
        'argocd-port-forward',
        serve_cmd='kubectl port-forward -n argocd deployment/argocd-server 8080:8080',
        labels=['platform'],
        auto_init=False,
        readiness_probe=probe(
            period_secs=5,
            exec=exec_action(['sh', '-c', 'nc -z localhost 8080'])
        )
    )

# =============================================================================
# Utility Commands
# =============================================================================
local_resource(
    'check-rust',
    cmd='cargo check --workspace',
    labels=['utilities'],
    auto_init=False,
    allow_parallel=True,
)

local_resource(
    'test-rust',
    cmd='cargo test --workspace',
    labels=['utilities'],
    auto_init=False,
    allow_parallel=True,
)



