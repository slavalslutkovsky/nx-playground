"""
GCP Cloud SQL resource generation for database composition.
"""

import schemas.database as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.common as c

generate = lambda spec: schema.DatabaseSpec, ocds: any, name: str -> [any] {
    """Generate GCP Cloud SQL resources from DatabaseSpec."""
    _db_name = name
    _region = c.map_region("gcp", spec.region) if spec.region else ""
    _tier = schema.get_instance_class("gcp", spec.size)
    _database_version = _get_gcp_database_version(spec.engine, spec.engineVersion)

    _label_config = labels.LabelConfig {
        name = _db_name
        component = "database"
        provider = "gcp"
        environment = spec.environment
    }
    _user_labels = labels.generate_tags(_label_config, "gcp") | {
        k.lower().replace("-", "_"): v.lower()
        for k, v in spec.tags
        if len(k) <= 63 and len(v) <= 63
    }
    _k8s_labels = labels.generate_labels(_label_config)

    resources: [any] = []

    # Build settings
    _settings: {str:any} = {
        tier = _tier
        diskSize = spec.storageGB
        diskType = "PD_SSD"
        diskAutoresize = True
        diskAutoresizeLimit = spec.storageGB * 2
        availabilityType = "REGIONAL" if spec.multiAZ else "ZONAL"
        userLabels = _user_labels
        ipConfiguration = {
            ipv4Enabled = spec.publiclyAccessible
            requireSsl = True
        }
    }

    # Add backup configuration
    if spec.backup.enabled:
        _settings |= {
            backupConfiguration = {
                enabled = True
                startTime = spec.backup.preferredWindow.split("-")[0]
                pointInTimeRecoveryEnabled = True
                transactionLogRetentionDays = spec.backup.retentionDays
                backupRetentionSettings = {
                    retainedBackups = spec.backup.retentionDays
                    retentionUnit = "COUNT"
                }
            }
        }

    # Main Cloud SQL Instance
    _sql_spec: {str:any} = {
        forProvider = {
            region = _region
            databaseVersion = _database_version
            deletionProtection = spec.environment == "prod"
            settings = _settings
        }
        deletionPolicy = spec.deletionPolicy
    }

    # Add write connection secret if auto-generating password
    if spec.secrets.generatePassword:
        _sql_spec |= {
            writeConnectionSecretToRef = {
                name = "${_db_name}-credentials"
                namespace = "crossplane-system"
            }
        }

    resources += [{
        apiVersion = "sql.gcp.upbound.io/v1beta2"
        kind = "DatabaseInstance"
        metadata = h.metadata_with_labels("${_db_name}-cloudsql", _k8s_labels)
        spec = _sql_spec
    }]

    # Create database
    resources += [{
        apiVersion = "sql.gcp.upbound.io/v1beta1"
        kind = "Database"
        metadata = h.metadata_with_labels("${_db_name}-db", _k8s_labels)
        spec = {
            forProvider = {
                instanceSelector = {
                    matchControllerRef = True
                }
                charset = "UTF8" if spec.engine == "postgres" else "utf8mb4"
                collation = "en_US.UTF8" if spec.engine == "postgres" else "utf8mb4_general_ci"
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # Create user
    _user_spec: {str:any} = {
        forProvider = {
            instanceSelector = {
                matchControllerRef = True
            }
            $type = "BUILT_IN"
        }
        deletionPolicy = spec.deletionPolicy
    }

    if spec.secrets.generatePassword:
        _user_spec.forProvider |= {
            passwordSecretRef = {
                name = "${_db_name}-user-password"
                namespace = "crossplane-system"
                key = "password"
            }
        }
    elif spec.secrets.passwordSecretRef:
        _user_spec.forProvider |= {
            passwordSecretRef = {
                name = spec.secrets.passwordSecretRef
                namespace = "crossplane-system"
                key = "password"
            }
        }

    resources += [{
        apiVersion = "sql.gcp.upbound.io/v1beta1"
        kind = "User"
        metadata = h.metadata_with_labels("${_db_name}-user", _k8s_labels)
        spec = _user_spec
    }]

    resources
}

_get_gcp_database_version = lambda engine: str, version: str -> str {
    """Get the GCP Cloud SQL database version string."""
    if engine == "postgres":
        "POSTGRES_${version}"
    else:
        "MYSQL_8_0"
}

get_status = lambda spec: schema.DatabaseSpec, ocds: any, name: str -> schema.DatabaseStatus {
    """Extract status from observed composed resources."""
    _db_name = name
    _observed_db = ocds.get("${_db_name}-cloudsql", {})
    _observed_status = _observed_db.get("Resource", {}).get("status", {}).get("atProvider", {})

    _connection_name = _observed_status.get("connectionName", "")

    schema.DatabaseStatus {
        provider = "gcp"
        region = spec.region
        engine = spec.engine
        endpoint = _connection_name
    }
}
