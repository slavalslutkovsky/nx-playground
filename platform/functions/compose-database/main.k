"""
Multi-provider database composition.
Creates AWS RDS, GCP CloudSQL, or CloudNative-PG based on provider field.
Uses modular provider implementations for clean separation.
"""

import schemas.database as schema
import .providers.aws as aws
import .providers.gcp as gcp
import .providers.kubernetes as k8s

# Crossplane function inputs
oxr = option("params").oxr
_ocds = option("params").ocds or {}
dxr = option("params").dxr
dcds = option("params").dcds

# Get resource name from oxr metadata
_name = oxr.metadata.name

# Type-annotated spec extraction
_raw_spec = oxr.spec

# Parse backup config
_backup = schema.BackupConfig {
    enabled = _raw_spec.backup.enabled if _raw_spec.backup and _raw_spec.backup.enabled != None else True
    retentionDays = _raw_spec.backup.retentionDays if _raw_spec.backup and _raw_spec.backup.retentionDays else 7
    preferredWindow = _raw_spec.backup.preferredWindow if _raw_spec.backup and _raw_spec.backup.preferredWindow else "03:00-04:00"
} if _raw_spec.backup else schema.BackupConfig {}

# Parse secrets config
_secrets = schema.SecretsConfig {
    generatePassword = _raw_spec.secrets.generatePassword if _raw_spec.secrets and _raw_spec.secrets.generatePassword != None else True
    passwordSecretRef = _raw_spec.secrets.passwordSecretRef if _raw_spec.secrets and _raw_spec.secrets.passwordSecretRef else ""
    usernameSecretRef = _raw_spec.secrets.usernameSecretRef if _raw_spec.secrets and _raw_spec.secrets.usernameSecretRef else ""
} if _raw_spec.secrets else schema.SecretsConfig {}

_spec: schema.DatabaseSpec = schema.DatabaseSpec {
    provider = _raw_spec.provider
    region = _raw_spec.region or ""
    engine = _raw_spec.engine or "postgres"
    engineVersion = _raw_spec.engineVersion or "18"
    size = _raw_spec.size or "small"
    storageGB = _raw_spec.storageGB or 20
    storageType = _raw_spec.storageType or "gp3"
    multiAZ = _raw_spec.multiAZ or False
    publiclyAccessible = _raw_spec.publiclyAccessible or False
    instances = _raw_spec.instances or 1
    namespace = _raw_spec.namespace or "default"
    environment = _raw_spec.environment or "dev"
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
    tags = _raw_spec.tags or {}
    backup = _backup
    secrets = _secrets
}

# Generate provider-specific resources
_resources = if _spec.provider == "aws":
    aws.generate(_spec, _ocds)
elif _spec.provider == "gcp":
    gcp.generate(_spec, _ocds, _name)
else:
    k8s.generate(_spec, _ocds, _name)

# Get provider-specific status
_status = if _spec.provider == "aws":
    aws.get_status(_spec, _ocds)
elif _spec.provider == "gcp":
    gcp.get_status(_spec, _ocds, _name)
else:
    k8s.get_status(_spec, _ocds, _name)

# Update composite resource status
_dxr = {
    **oxr
    status = {
        provider = _status.provider
        region = _status.region
        engine = _status.engine
        endpoint = _status.endpoint
    }
}

# Output items
items = _resources + [_dxr]
