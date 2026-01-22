"""
Multi-provider database composition.
Creates AWS RDS, GCP CloudSQL, or CloudNative-PG based on provider field.

Uses typed KCL schemas from Upbound provider modules where available.
"""

import schemas.database as schema
import schemas.helpers as h
import schemas.mappings as m
import schemas.regions as regions

# Import typed Crossplane resource schemas
import crossplane_provider_upjet_aws.v1beta2.rds_aws_upbound_io_v1beta2_instance as rds

# Crossplane function inputs
oxr = option("params").oxr
_ocds = option("params").ocds
dxr = option("params").dxr
dcds = option("params").dcds

# Type-annotated spec extraction
_raw_spec = oxr.spec
_spec: schema.DatabaseSpec = schema.DatabaseSpec {
    provider = _raw_spec.provider
    region = _raw_spec.region or ""
    engine = _raw_spec.engine or "postgres"
    engineVersion = _raw_spec.engineVersion or "15"
    size = _raw_spec.size or "small"
    storageGB = _raw_spec.storageGB or 20
    instances = _raw_spec.instances or 1
    namespace = _raw_spec.namespace or "default"
    environment = _raw_spec.environment or "dev"
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
}

# Use typed spec
_name = oxr.metadata.name
_provider = _spec.provider
_region = _spec.region
_engine = _spec.engine
_engine_version = _spec.engineVersion
_size = _spec.size
_storage_gb = _spec.storageGB
_instances = _spec.instances
_namespace = _spec.namespace
_env = _spec.environment
_deletion_policy = _spec.deletionPolicy

# Get provider-specific region
_aws_region = regions.get_aws_region(_region) if _region else ""
_gcp_region = regions.get_gcp_region(_region) if _region else ""

# AWS RDS Instance using typed schema
_aws_db = rds.Instance {
    metadata = h.metadata("${_name}-rds")
    spec = {
        forProvider = {
            region = _aws_region
            engine = _engine
            engineVersion = _engine_version
            instanceClass = m.get_aws_rds_instance_class(_size)
            allocatedStorage = _storage_gb
            skipFinalSnapshot = True if _env == "dev" else False
            publiclyAccessible = False
            tags = h.aws_tags(_env)
        }
        deletionPolicy = _deletion_policy
    }
}

# GCP CloudSQL Instance (raw dict - KCL module only has v1beta1, provider has v1beta2)
# TODO: Update when crossplane-provider-upjet-gcp adds v1beta2 support
_gcp_db = {
    apiVersion = "sql.gcp.upbound.io/v1beta2"
    kind = "DatabaseInstance"
    metadata = h.metadata("${_name}-cloudsql")
    spec = {
        forProvider = {
            region = _gcp_region
            databaseVersion = "POSTGRES_${_engine_version}" if _engine == "postgres" else "MYSQL_8_0"
            settings = {
                tier = m.get_gcp_cloudsql_tier(_size)
                diskSize = _storage_gb
                diskType = "PD_SSD"
                ipConfiguration = {
                    ipv4Enabled = True
                }
                userLabels = h.gcp_labels(_env)
            }
            deletionProtection = False if _env == "dev" else True
        }
        deletionPolicy = _deletion_policy
    }
}

# CloudNative-PG Cluster (for kubernetes provider - no typed module available)
_cnpg_db = {
    apiVersion = "postgresql.cnpg.io/v1"
    kind = "Cluster"
    metadata = {
        name = "${_name}-cnpg"
        namespace = _namespace
        annotations = {
            "krm.kcl.dev/composition-resource-name" = "${_name}-cnpg"
        }
        labels = h.k8s_labels(_env)
    }
    spec = {
        instances = _instances
        imageName = "ghcr.io/cloudnative-pg/postgresql:${_engine_version}"
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
            size = "${_storage_gb}Gi"
        }
        resources = m.get_cnpg_resources(_size)
        monitoring = {
            enablePodMonitor = True if _env != "dev" else False
        }
    }
}

# Select database based on provider
_db = _aws_db if _provider == "aws" else (_gcp_db if _provider == "gcp" else _cnpg_db)

# Update composite resource status
_status: schema.DatabaseStatus = schema.DatabaseStatus {
    provider = _provider
    region = _region if _provider != "kubernetes" else "in-cluster"
    engine = _engine
}

_dxr = {
    **oxr
    status = _status
}

# Output items
items = [_db, _dxr]
