#k8s_yaml(kustomize('./manifests/cnpg/base'))
k8s_yaml(kustomize('./manifests/k8s/overlays/dev'))

# =============================================================================
# Database Port Forwards
# =============================================================================
local_resource(
    'postgres',
    serve_cmd='kubectl port-forward -n dbs deployment/postgres 5432:5432',
    labels=['port-forward'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 5432'])
    )
)



local_resource(
    'redis',
    serve_cmd='kubectl port-forward -n dbs deployment/redis 6379:6379',
    labels=['port-forward'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 6379'])
    )
)

local_resource(
    'mailhog',
    serve_cmd='kubectl port-forward -n dbs deployment/mailhog 8025:8025',
    labels=['port-forward'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 8025'])
    )
)

local_resource(
    'istio-gateway',
    serve_cmd='kubectl port-forward -n gateway svc/main-gateway-istio 8080:80 8443:443',
    labels=['port-forward'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 8080'])
    )
)
# =============================================================================
# Schema ConfigMap Generation
# Regenerates and applies the schema ConfigMap when schema.sql changes
# =============================================================================
local_resource(
    'schema-configmap',
    cmd='just gen-schema-configmap',
    labels=['migrations'],
    deps=[
        'manifests/schemas/schema.sql',
    ],
)

# =============================================================================
# Database Setup - Seed Data Only
# Atlas operator handles schema via AtlasSchema CR
# =============================================================================
local_resource(
    'db-seed',
    cmd='''
        echo "Applying seed data..."
        kubectl exec -i -n dbs deployment/postgres -- psql -U myuser -d mydatabase < manifests/schemas/seed.sql
        echo "Seed data applied!"
    ''',
    labels=['migrations'],
    resource_deps=['postgres-port-forward'],
    deps=[
        'manifests/schemas/seed.sql',
    ],
)

# =============================================================================
# Applications
# =============================================================================
include('./apps/zerg/api/Tiltfile')
include('./apps/zerg/tasks/Tiltfile')
include('./apps/zerg/web/Tiltfile')
include('./apps/zerg/email-nats/Tiltfile')
