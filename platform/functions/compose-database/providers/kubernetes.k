"""
Kubernetes CloudNative-PG resource generation for database composition.
"""

import schemas.database as schema
import schemas.labels as labels
import schemas.helpers as h

generate = lambda spec: schema.DatabaseSpec, ocds: any, name: str -> [any] {
    """Generate CloudNative-PG Cluster from DatabaseSpec."""
    _db_name = name
    _namespace = spec.namespace
    _resources = schema.get_instance_class("kubernetes", spec.size)

    _label_config = labels.LabelConfig {
        name = _db_name
        component = "database"
        provider = "kubernetes"
        environment = spec.environment
    }
    _k8s_labels = labels.generate_labels(_label_config)

    resources: [any] = []

    # CloudNative-PG Cluster
    _cnpg_spec: {str:any} = {
        instances = spec.instances
        imageName = "ghcr.io/cloudnative-pg/postgresql:${spec.engineVersion}"
        postgresql = {
            parameters = {
                max_connections = "100"
                shared_buffers = "128MB"
            }
        }
        bootstrap = {
            initdb = {
                database = "app"
                owner = "app"
            }
        }
        storage = {
            size = "${spec.storageGB}Gi"
        }
        resources = {
            requests = _resources
            limits = _resources
        }
        monitoring = {
            enablePodMonitor = spec.environment != "dev"
        }
    }

    # Add backup configuration if enabled
    if spec.backup.enabled:
        _cnpg_spec |= {
            backup = {
                barmanObjectStore = {
                    destinationPath = "s3://${_db_name}-backups/"
                    s3Credentials = {
                        accessKeyId = {
                            name = "${_db_name}-backup-creds"
                            key = "ACCESS_KEY_ID"
                        }
                        secretAccessKey = {
                            name = "${_db_name}-backup-creds"
                            key = "SECRET_ACCESS_KEY"
                        }
                    }
                }
                retentionPolicy = "${spec.backup.retentionDays}d"
            }
        }

    resources += [{
        apiVersion = "postgresql.cnpg.io/v1"
        kind = "Cluster"
        metadata = {
            name = "${_db_name}-cnpg"
            namespace = _namespace
            annotations = {
                "krm.kcl.dev/composition-resource-name" = "${_db_name}-cnpg"
            }
            labels = _k8s_labels
        }
        spec = _cnpg_spec
    }]

    resources
}

get_status = lambda spec: schema.DatabaseSpec, ocds: any, name: str -> schema.DatabaseStatus {
    """Extract status from observed composed resources."""
    _db_name = name
    _observed_db = ocds.get("${_db_name}-cnpg", {})
    _observed_status = _observed_db.get("Resource", {}).get("status", {}) or {}

    _instances_ready = _observed_status.get("instancesReady", 0)

    schema.DatabaseStatus {
        provider = "kubernetes"
        region = "in-cluster"
        engine = spec.engine
        endpoint = "${_db_name}-cnpg-rw.${spec.namespace}.svc.cluster.local:5432" if _instances_ready > 0 else ""
    }
}
