"""
AWS RDS resource generation for database module.
"""

import ...common.types
import ...common.labels
import ..schema

generate = lambda spec: schema.DatabaseSpec -> [any] {
    """Generate AWS RDS resources from DatabaseSpec."""
    _db_name = spec.name
    _region = spec.region
    _instance_class = schema.get_instance_class("aws", spec.size)
    _engine_version = schema.get_engine_version(spec.engine, "aws", spec.engineVersion)

    _label_config = labels.LabelConfig {
        name = _db_name
        component = "database"
        provider = "aws"
        environment = spec.environment
    }
    _tags = labels.generate_tags(_label_config, "aws") | spec.tags
    _labels = labels.generate_labels(_label_config)

    resources = []

    # Subnet Group (required for RDS)
    resources += [{
        apiVersion: "rds.aws.upbound.io/v1beta1"
        kind: "SubnetGroup"
        metadata: {
            name: "${_db_name}-subnet-group"
            labels: _labels
        }
        spec: {
            forProvider: {
                region: _region
                description: "Subnet group for ${_db_name}"
                # In a real setup, these would come from a VPC module
                subnetIdSelector: {
                    matchLabels: {
                        "crossplane.io/network": "default"
                        "crossplane.io/visibility": "private" if not spec.publiclyAccessible else "public"
                    }
                }
                tags: _tags
            }
            deletionPolicy: spec.deletionPolicy
        }
    }]

    # Parameter Group
    _family = _get_parameter_family(spec.engine, _engine_version)
    resources += [{
        apiVersion: "rds.aws.upbound.io/v1beta1"
        kind: "ParameterGroup"
        metadata: {
            name: "${_db_name}-params"
            labels: _labels
        }
        spec: {
            forProvider: {
                region: _region
                family: _family
                description: "Parameter group for ${_db_name}"
                tags: _tags
            }
            deletionPolicy: spec.deletionPolicy
        }
    }]

    # Main RDS Instance
    rds_instance = {
        apiVersion: "rds.aws.upbound.io/v1beta1"
        kind: "Instance"
        metadata: {
            name: _db_name
            labels: _labels
        }
        spec: {
            forProvider: {
                region: _region
                engine: spec.engine
                engineVersion: _engine_version
                instanceClass: _instance_class
                allocatedStorage: spec.storageGB
                storageType: spec.storageType
                storageEncrypted: True
                multiAz: spec.multiAZ
                publiclyAccessible: spec.publiclyAccessible
                dbSubnetGroupNameSelector: {
                    matchLabels: {
                        "${labels.LABEL_APP}": _db_name
                    }
                }
                parameterGroupNameSelector: {
                    matchLabels: {
                        "${labels.LABEL_APP}": _db_name
                    }
                }
                skipFinalSnapshot: spec.environment != "prod"
                finalSnapshotIdentifier: "${_db_name}-final" if spec.environment == "prod" else None
                copyTagsToSnapshot: True
                tags: _tags
            }
            deletionPolicy: spec.deletionPolicy
        }
    }

    # Add backup configuration
    if spec.backup.enabled:
        rds_instance.spec.forProvider |= {
            backupRetentionPeriod: spec.backup.retentionDays
            backupWindow: spec.backup.preferredWindow
        }

    # Add password configuration
    if spec.secrets.generatePassword:
        rds_instance.spec.forProvider |= {
            username: "admin"
            autoGeneratePassword: True
            passwordSecretRef: None
        }
        rds_instance.spec |= {
            writeConnectionSecretToRef: {
                name: "${_db_name}-credentials"
                namespace: "crossplane-system"
            }
        }
    elif spec.secrets.passwordSecretRef:
        rds_instance.spec.forProvider |= {
            username: "admin"
            passwordSecretRef: {
                name: spec.secrets.passwordSecretRef
                namespace: "crossplane-system"
                key: "password"
            }
        }

    resources += [rds_instance]

    resources
}

_get_parameter_family = lambda engine: str, version: str -> str {
    """Get the RDS parameter group family."""
    if engine == "postgres":
        major = version.split(".")[0]
        "postgres${major}"
    elif engine == "mysql":
        major_minor = ".".join(version.split(".")[:2])
        "mysql${major_minor}"
    else:
        "postgres15"
}
