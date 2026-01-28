"""
Kubernetes Harbor resource generation for registry composition.
"""

import schemas.registry as schema
import schemas.labels as labels
import schemas.helpers as h

generate = lambda spec: schema.RegistrySpec, ocds: any -> [any] {
    """Generate Harbor resources from RegistrySpec."""
    _name = spec.name
    _namespace = spec.namespace

    _label_config = labels.LabelConfig {
        name = _name
        component = "registry"
        provider = "kubernetes"
        environment = spec.environment
    }
    _k8s_labels = labels.generate_labels(_label_config)

    resources: [any] = []

    # Harbor Cluster (using goharbor-operator)
    resources += [{
        apiVersion = "goharbor.io/v1beta1"
        kind = "HarborCluster"
        metadata = {
            name = "${_name}-harbor"
            namespace = _namespace
            annotations = {
                "krm.kcl.dev/composition-resource-name" = "${_name}-harbor"
            }
            labels = _k8s_labels
        }
        spec = {
            version = "2.11.0"
            externalURL = "https://${_name}-harbor.${_namespace}.svc.cluster.local"
            expose = {
                core = {
                    ingress = {
                        host = "${_name}-harbor.local"
                    }
                }
            }
            database = {
                kind = "PostgreSQL"
                spec = {
                    embedded = {}
                }
            }
            storage = {
                kind = "FileSystem"
                spec = {
                    fileSystem = {
                        rootdirectory = "/storage"
                    }
                }
            }
            cache = {
                kind = "Redis"
                spec = {
                    embedded = {}
                }
            }
        }
    }]

    resources
}

get_status = lambda spec: schema.RegistrySpec, ocds: any -> schema.RegistryStatus {
    """Extract status from observed composed resources."""
    _observed_harbor = ocds.get("${spec.name}-harbor", {})
    _observed_status = _observed_harbor.get("Resource", {}).get("status", {}) or {}

    _external_url = _observed_status.get("externalURL", "")

    schema.RegistryStatus {
        provider = "kubernetes"
        region = "in-cluster"
        registryUrl = _external_url if _external_url else "https://${spec.name}-harbor.${spec.namespace}.svc.cluster.local"
        repositoryName = spec.name
    }
}
