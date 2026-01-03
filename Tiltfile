#k8s_yaml(kustomize('./manifests/kustomize/domains/terran/overlays/dev'))
#k8s_yaml(kustomize('./manifests/kustomize/domains/zerg/overlays/dev'))
#k8s_yaml(kustomize('./manifests/kustomize/all/dev'))

include('./apps/zerg/api/Tiltfile')
include('./apps/zerg/tasks/Tiltfile')
include('./apps/zerg/web/Tiltfile')
#include('./apps/zerg/operator/Tiltfile')

#include('./apps/cargo-docs/Tiltfile')
#manifests/kustomize/all/dev
#include('./apps/terran/api/Tiltfile')
#include('./apps/terran/web/Tiltfile')

local_resource(
    'redis-port-forward',
    serve_cmd='kubectl port-forward -n dbs deployment/redis 6379:6379',
    labels=['databases'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 6379'])
    )
)

local_resource(
    'postgres-port-forward',
    serve_cmd='kubectl port-forward -n dbs deployment/postgres 5433:5432',
    labels=['databases'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 5433'])
    )
)

# local_resource(
#    'mongodb-port-forward',
#    serve_cmd='kubectl port-forward -n dbs deployment/db 27017:27017',
#    labels=['databases'],
#    readiness_probe=probe(
#        period_secs=5,
#        exec=exec_action(['sh', '-c', 'nc -z localhost 27017'])
#    )
# )

local_resource(
    'influxdb2-port-forward',
    serve_cmd='kubectl port-forward -n dbs deployment/influxdb2 8086:8086',
    labels=['databases'],
    readiness_probe=probe(
        period_secs=5,
        exec=exec_action(['sh', '-c', 'nc -z localhost 8086'])
    )
)

# local_resource(
#     'komoplane-port-forward',
#     serve_cmd='kubectl port-forward -n crossplane-system deployment/komoplane 8090:8090',
#     labels=['databases'],
#     readiness_probe=probe(
#         period_secs=5,
#         exec=exec_action(['sh', '-c', 'nc -z localhost 8090'])
#     )
# )

# local_resource(
#     'grafana-port-forward',
#     serve_cmd='kubectl port-forward -n monitoring deployment/monitoring-grafana 3000:3000',
#     labels=['databases'],
#     readiness_probe=probe(
#         period_secs=5,
#         exec=exec_action(['sh', '-c', 'nc -z localhost 3000'])
#     )
# )

# local_resource(
#     'argocd-port-forward',
#     serve_cmd='kubectl port-forward -n argocd deployment/argocd-server 8080:8080',
#     labels=['databases'],
#     readiness_probe=probe(
#         period_secs=5,
#         exec=exec_action(['sh', '-c', 'nc -z localhost 8080'])
#     )
# )

#k8s_yaml(kustomize('./manifests/kustomize/backstage/overlays/dev'))

#local_resource(
#    'backstage-port-forward',
#    serve_cmd='kubectl port-forward -n backstage deployment/backstage 7007:7007',
#    labels=['platform'],
#    readiness_probe=probe(
#        period_secs=5,
#        exec=exec_action(['sh', '-c', 'nc -z localhost 7007'])
#    )
#)
