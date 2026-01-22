"""
Multi-provider container registry composition.
Creates AWS ECR, GCP Artifact Registry, or Harbor based on provider field.

Uses typed KCL schemas from Upbound provider modules for type safety.
"""

import schemas.registry as schema
import schemas.helpers as h
import schemas.regions as regions

# Import typed Crossplane resource schemas
import crossplane_provider_upjet_aws.v1beta2.ecr_aws_upbound_io_v1beta2_repository as ecr
import crossplane_provider_upjet_aws.v1beta1.ecr_aws_upbound_io_v1beta1_lifecycle_policy as ecr_lifecycle
import crossplane_provider_upjet_gcp.v1beta1.artifact_gcp_upbound_io_v1beta1_registry_repository as artifact

# Crossplane function inputs
oxr = option("params").oxr
_ocds = option("params").ocds
dxr = option("params").dxr
dcds = option("params").dcds

# Type-annotated spec extraction
_raw_spec = oxr.spec

# Handle optional boolean fields with defaults
_scan_on_push_raw = _raw_spec.scanOnPush
_encryption_raw = _raw_spec.encryption

_spec: schema.RegistrySpec = schema.RegistrySpec {
    provider = _raw_spec.provider
    name = _raw_spec.name
    region = _raw_spec.region or ""
    format = _raw_spec.format or "docker"
    immutableTags = _raw_spec.immutableTags or False
    scanOnPush = _scan_on_push_raw if _scan_on_push_raw != Undefined else True
    encryption = _encryption_raw if _encryption_raw != Undefined else True
    publicAccess = _raw_spec.publicAccess or False
    retentionDays = _raw_spec.retentionDays or 0
    namespace = _raw_spec.namespace or "default"
    environment = _raw_spec.environment or "dev"
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
}

# Use typed spec
_name = _spec.name
_provider = _spec.provider
_region = _spec.region
_format = _spec.format
_immutable_tags = _spec.immutableTags
_scan_on_push = _spec.scanOnPush
_public_access = _spec.publicAccess
_retention_days = _spec.retentionDays
_namespace = _spec.namespace
_env = _spec.environment
_deletion_policy = _spec.deletionPolicy

# Get provider-specific region
_aws_region = regions.get_aws_region(_region) if _region else ""
_gcp_region = regions.get_gcp_region(_region) if _region else ""

# AWS ECR Repository using typed schema
_aws_ecr = ecr.Repository {
    metadata = h.metadata("${_name}-ecr")
    spec = {
        forProvider = {
            region = _aws_region
            imageTagMutability = "IMMUTABLE" if _immutable_tags else "MUTABLE"
            imageScanningConfiguration = {
                scanOnPush = _scan_on_push
            }
            encryptionConfiguration = [{
                encryptionType = "AES256"
            }]
            tags = h.aws_tags(_env)
        }
        deletionPolicy = _deletion_policy
    }
}

# AWS ECR Lifecycle Policy using typed schema (for retention)
_aws_ecr_lifecycle = ecr_lifecycle.LifecyclePolicy {
    metadata = h.metadata("${_name}-ecr-lifecycle")
    spec = {
        forProvider = {
            region = _aws_region
            repositorySelector = {
                matchControllerRef = True
            }
            policy = '''{
                "rules": [
                    {
                        "rulePriority": 1,
                        "description": "Expire untagged images older than ''' + str(_retention_days) + ''' days",
                        "selection": {
                            "tagStatus": "untagged",
                            "countType": "sinceImagePushed",
                            "countUnit": "days",
                            "countNumber": ''' + str(_retention_days) + '''
                        },
                        "action": {
                            "type": "expire"
                        }
                    }
                ]
            }'''
        }
    }
} if _retention_days > 0 else None

# GCP Artifact Registry Repository using typed schema
# Note: Uses artifact.gcp.upbound.io/v1beta1, not artifactregistry.gcp.upbound.io
_gcp_artifact_registry = artifact.RegistryRepository {
    metadata = h.metadata("${_name}-artifactregistry")
    spec = {
        forProvider = {
            location = _gcp_region
            format = _format.upper()
            mode = "STANDARD_REPOSITORY"
            labels = h.gcp_labels(_env)
        }
        deletionPolicy = _deletion_policy
    }
}

# Harbor instance for kubernetes provider (no typed module available)
_harbor = {
    apiVersion = "goharbor.io/v1beta1"
    kind = "HarborCluster"
    metadata = {
        name = "${_name}-harbor"
        namespace = _namespace
        annotations = {
            "krm.kcl.dev/composition-resource-name" = "${_name}-harbor"
        }
        labels = h.k8s_labels(_env)
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
}

# Select registry based on provider
_registry = _aws_ecr if _provider == "aws" else (_gcp_artifact_registry if _provider == "gcp" else _harbor)

# Build items list (use any type to allow mixed resource types)
_items: [any] = [_registry]

# Add lifecycle policy for ECR if retention is set
if _provider == "aws" and _aws_ecr_lifecycle:
    _items += [_aws_ecr_lifecycle]

# Update composite resource status
_status: schema.RegistryStatus = schema.RegistryStatus {
    provider = _provider
    region = _region if _provider != "kubernetes" else "in-cluster"
    repositoryName = _name
}

_dxr = {
    **oxr
    status = _status
}

_items += [_dxr]

# Output items
#items = _items + [data]
ds = data
items = _items
