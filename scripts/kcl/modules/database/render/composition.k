"""
Composition generator for database module.
Generates the Composition YAML that uses function-kcl.
Run: kcl run scripts/kcl/modules/database/render/composition.k -o yaml
"""

# KCL source for the composition pipeline
_kcl_source = '''
import .schema
import .providers.aws as aws
import .providers.gcp as gcp

oxr = option("params").oxr
ocds = option("params").ocds or {}

_spec = schema.DatabaseSpec {
    provider = oxr.spec.provider
    environment = oxr.spec.get("environment", "dev")
    region = oxr.spec.region
    deletionPolicy = oxr.spec.get("deletionPolicy", "Delete")
    tags = oxr.spec.get("tags", {})
    name = oxr.spec.name
    engine = oxr.spec.get("engine", "postgres")
    engineVersion = oxr.spec.get("engineVersion", "")
    size = oxr.spec.get("size", "small")
    storageGB = oxr.spec.get("storageGB", 20)
    storageType = oxr.spec.get("storageType", "gp3")
    multiAZ = oxr.spec.get("multiAZ", False)
    publiclyAccessible = oxr.spec.get("publiclyAccessible", False)
    backup = schema.BackupConfig {
        enabled = oxr.spec.get("backup", {}).get("enabled", True)
        retentionDays = oxr.spec.get("backup", {}).get("retentionDays", 7)
        preferredWindow = oxr.spec.get("backup", {}).get("preferredWindow", "03:00-04:00")
    }
    secrets = schema.SecretsConfig {
        generatePassword = oxr.spec.get("secrets", {}).get("generatePassword", True)
        passwordSecretRef = oxr.spec.get("secrets", {}).get("passwordSecretRef", "")
    }
}

_generate = lambda provider: str -> [any] {
    if provider == "aws":
        aws.generate(_spec)
    elif provider == "gcp":
        gcp.generate(_spec)
    else:
        []
}

_resources = _generate(_spec.provider)

dxr = {
    **oxr
    status: {
        provider: _spec.provider
        region: _spec.region
        engine: _spec.engine
        instanceName: _spec.name
        size: _spec.size
    }
}

items = _resources + [dxr]
'''

composition = {
    apiVersion: "apiextensions.crossplane.io/v1"
    kind: "Composition"
    metadata: {
        name: "database"
        labels: {
            "crossplane.io/module": "database"
            "crossplane.io/xrd": "xdatabases.database.platform.io"
        }
    }
    spec: {
        compositeTypeRef: {
            apiVersion: "database.platform.io/v1alpha1"
            kind: "XDatabase"
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
