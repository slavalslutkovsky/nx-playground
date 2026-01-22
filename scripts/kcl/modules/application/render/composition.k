"""
Composition generator for application module.
Generates the Composition YAML that uses function-kcl.
Run: kcl run scripts/kcl/modules/application/render/composition.k -o yaml
"""

# KCL source for the composition pipeline
_kcl_source = '''
import .schema

oxr = option("params").oxr
ocds = option("params").ocds or {}

_spec = schema.ApplicationSpec {
    provider = oxr.spec.provider
    environment = oxr.spec.get("environment", "dev")
    region = oxr.spec.region
    deletionPolicy = oxr.spec.get("deletionPolicy", "Delete")
    tags = oxr.spec.get("tags", {})
    name = oxr.spec.name
    storage = schema.StorageConfig {
        enabled = oxr.spec.get("storage", {}).get("enabled", False)
        storageClass = oxr.spec.get("storage", {}).get("storageClass", "standard")
        versioning = oxr.spec.get("storage", {}).get("versioning", False)
        encryption = oxr.spec.get("storage", {}).get("encryption", True)
    } if oxr.spec.get("storage") else None
    database = schema.DatabaseConfig {
        enabled = oxr.spec.get("database", {}).get("enabled", False)
        engine = oxr.spec.get("database", {}).get("engine", "postgres")
        size = oxr.spec.get("database", {}).get("size", "small")
        storageGB = oxr.spec.get("database", {}).get("storageGB", 20)
        multiAZ = oxr.spec.get("database", {}).get("multiAZ", False)
        backup = schema.BackupConfig {
            enabled = oxr.spec.get("database", {}).get("backup", {}).get("enabled", True)
            retentionDays = oxr.spec.get("database", {}).get("backup", {}).get("retentionDays", 7)
        }
    } if oxr.spec.get("database") else None
}

_resources = []

if _spec.storage?.enabled:
    _resources += [{
        apiVersion: "storage.platform.io/v1alpha1"
        kind: "XBucket"
        metadata: {
            name: "${_spec.name}-storage"
            labels: {
                "app.kubernetes.io/name": _spec.name
                "app.kubernetes.io/component": "storage"
                "crossplane.io/composite": oxr.metadata.name
            }
        }
        spec: {
            provider: _spec.provider
            region: _spec.region
            environment: _spec.environment
            deletionPolicy: _spec.deletionPolicy
            name: "${_spec.name}-data"
            storageClass: _spec.storage.storageClass
            versioning: _spec.storage.versioning
            encryption: _spec.storage.encryption
            tags: _spec.tags | {"app.kubernetes.io/part-of": _spec.name}
        }
    }]

if _spec.database?.enabled:
    _resources += [{
        apiVersion: "database.platform.io/v1alpha1"
        kind: "XDatabase"
        metadata: {
            name: "${_spec.name}-db"
            labels: {
                "app.kubernetes.io/name": _spec.name
                "app.kubernetes.io/component": "database"
                "crossplane.io/composite": oxr.metadata.name
            }
        }
        spec: {
            provider: _spec.provider
            region: _spec.region
            environment: _spec.environment
            deletionPolicy: _spec.deletionPolicy
            name: "${_spec.name}-db"
            engine: _spec.database.engine
            size: _spec.database.size
            storageGB: _spec.database.storageGB
            multiAZ: _spec.database.multiAZ
            backup: {
                enabled: _spec.database.backup.enabled
                retentionDays: _spec.database.backup.retentionDays
            }
            tags: _spec.tags | {"app.kubernetes.io/part-of": _spec.name}
        }
    }]

dxr = {
    **oxr
    status: {
        provider: _spec.provider
        region: _spec.region
        applicationName: _spec.name
        storageEnabled: _spec.storage?.enabled or False
        databaseEnabled: _spec.database?.enabled or False
        storageName: "${_spec.name}-data" if _spec.storage?.enabled else ""
        databaseName: "${_spec.name}-db" if _spec.database?.enabled else ""
    }
}

items = _resources + [dxr]
'''

composition = {
    apiVersion: "apiextensions.crossplane.io/v1"
    kind: "Composition"
    metadata: {
        name: "application"
        labels: {
            "crossplane.io/module": "application"
            "crossplane.io/xrd": "xapplications.platform.io"
        }
    }
    spec: {
        compositeTypeRef: {
            apiVersion: "platform.io/v1alpha1"
            kind: "XApplication"
        }
        mode: "Pipeline"
        pipeline: [
            {
                step: "generate-resources"
                functionRef: {
                    name: "function-kcl"
                }
                input: {
                    apiVersion: "krm.kcl.dev/v1alpha1"
                    kind: "KCLRun"
                    spec: {
                        source: _kcl_source
                    }
                }
            }
            {
                step: "auto-ready"
                functionRef: {
                    name: "function-auto-ready"
                }
            }
        ]
    }
}

items = [composition]
