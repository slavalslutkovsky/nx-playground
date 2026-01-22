"""
GCP Cloud SQL resource generation for database module.
"""

import ...common.types
import ...common.labels
import ..schema

generate = lambda spec: schema.DatabaseSpec -> [any] {
    """Generate GCP Cloud SQL resources from DatabaseSpec."""
    _db_name = spec.name
    _region = types.map_region("gcp", spec.region)
    _tier = schema.get_instance_class("gcp", spec.size)
    _database_version = schema.get_engine_version(spec.engine, "gcp", spec.engineVersion)

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
    _labels = labels.generate_labels(_label_config)

    resources = []

    # Main Cloud SQL Instance
    sql_instance = {
        apiVersion: "sql.gcp.upbound.io/v1beta1"
        kind: "DatabaseInstance"
        metadata: {
            name: _db_name
            labels: _labels
        }
        spec: {
            forProvider: {
                region: _region
                databaseVersion: _database_version
                deletionProtection: spec.environment == "prod"
                settings: [{
                    tier: _tier
                    diskSize: spec.storageGB
                    diskType: "PD_SSD"
                    diskAutoresize: True
                    diskAutoresizeLimit: spec.storageGB * 2
                    availabilityType: "REGIONAL" if spec.multiAZ else "ZONAL"
                    userLabels: _user_labels
                    ipConfiguration: [{
                        ipv4Enabled: spec.publiclyAccessible
                        privateNetwork: None if spec.publiclyAccessible else ""
                        requireSsl: True
                    }]
                    backupConfiguration: [{
                        enabled: spec.backup.enabled
                        startTime: spec.backup.preferredWindow.split("-")[0]
                        pointInTimeRecoveryEnabled: spec.backup.enabled
                        transactionLogRetentionDays: spec.backup.retentionDays if spec.backup.enabled else 0
                        backupRetentionSettings: [{
                            retainedBackups: spec.backup.retentionDays
                            retentionUnit: "COUNT"
                        }]
                    }] if spec.backup.enabled else []
                }]
            }
            deletionPolicy: spec.deletionPolicy
        }
    }

    # Add write connection secret if auto-generating password
    if spec.secrets.generatePassword:
        sql_instance.spec |= {
            writeConnectionSecretToRef: {
                name: "${_db_name}-credentials"
                namespace: "crossplane-system"
            }
        }

    resources += [sql_instance]

    # Create database
    resources += [{
        apiVersion: "sql.gcp.upbound.io/v1beta1"
        kind: "Database"
        metadata: {
            name: "${_db_name}-db"
            labels: _labels
        }
        spec: {
            forProvider: {
                instanceSelector: {
                    matchLabels: {
                        "${labels.LABEL_APP}": _db_name
                    }
                }
                charset: "UTF8" if spec.engine == "postgres" else "utf8mb4"
                collation: "en_US.UTF8" if spec.engine == "postgres" else "utf8mb4_general_ci"
            }
            deletionPolicy: spec.deletionPolicy
        }
    }]

    # Create user
    user = {
        apiVersion: "sql.gcp.upbound.io/v1beta1"
        kind: "User"
        metadata: {
            name: "${_db_name}-user"
            labels: _labels
        }
        spec: {
            forProvider: {
                instanceSelector: {
                    matchLabels: {
                        "${labels.LABEL_APP}": _db_name
                    }
                }
                type: "BUILT_IN"
            }
            deletionPolicy: spec.deletionPolicy
        }
    }

    if spec.secrets.generatePassword:
        user.spec.forProvider |= {
            passwordSecretRef: {
                name: "${_db_name}-user-password"
                namespace: "crossplane-system"
                key: "password"
            }
        }
    elif spec.secrets.passwordSecretRef:
        user.spec.forProvider |= {
            passwordSecretRef: {
                name: spec.secrets.passwordSecretRef
                namespace: "crossplane-system"
                key: "password"
            }
        }

    resources += [user]

    resources
}
