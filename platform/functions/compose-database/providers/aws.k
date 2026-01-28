"""
AWS RDS resource generation for database composition.
"""

import crossplane_provider_upjet_aws.v1beta2.rds_aws_upbound_io_v1beta2_instance as rds
import schemas.database as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.regions as regions

generate = lambda spec: schema.DatabaseSpec, ocds: any -> [any] {
    """Generate AWS RDS resources from DatabaseSpec."""
    _db_name = spec.region  # use oxr metadata name
    _region = regions.get_aws_region(spec.region) if spec.region else ""
    _instance_class = schema.get_instance_class("aws", spec.size)
    _engine_version = schema.get_engine_version(spec.engine, "aws", spec.engineVersion)
    _storage_type = schema.get_storage_type("aws", spec.storageType)

    _label_config = labels.LabelConfig {
        name = _db_name
        component = "database"
        provider = "aws"
        environment = spec.environment
    }
    _tags = labels.generate_tags(_label_config, "aws") | spec.tags
    _k8s_labels = labels.generate_labels(_label_config)

    resources: [any] = []

    # Main RDS Instance using typed schema
    _rds_spec: {str:any} = {
        forProvider = {
            region = _region
            engine = spec.engine
            engineVersion = _engine_version
            instanceClass = _instance_class
            allocatedStorage = spec.storageGB
            storageType = _storage_type
            storageEncrypted = True
            multiAz = spec.multiAZ
            publiclyAccessible = spec.publiclyAccessible
            skipFinalSnapshot = spec.environment != "prod"
            copyTagsToSnapshot = True
            tags = _tags
        }
        deletionPolicy = spec.deletionPolicy
    }

    # Add final snapshot identifier for prod
    if spec.environment == "prod":
        _rds_spec.forProvider |= {
            finalSnapshotIdentifier = "${_db_name}-final"
        }

    # Add backup configuration
    if spec.backup.enabled:
        _rds_spec.forProvider |= {
            backupRetentionPeriod = spec.backup.retentionDays
            backupWindow = spec.backup.preferredWindow
        }

    # Add password configuration
    if spec.secrets.generatePassword:
        _rds_spec.forProvider |= {
            username = "admin"
            autoGeneratePassword = True
        }
        _rds_spec |= {
            writeConnectionSecretToRef = {
                name = "${_db_name}-credentials"
                namespace = "crossplane-system"
            }
        }
    elif spec.secrets.passwordSecretRef:
        _rds_spec.forProvider |= {
            username = "admin"
            passwordSecretRef = {
                name = spec.secrets.passwordSecretRef
                namespace = "crossplane-system"
                key = "password"
            }
        }

    resources += [rds.Instance {
        metadata = h.metadata_with_labels("${_db_name}-rds", _k8s_labels)
        spec = _rds_spec
    }]

    resources
}

get_status = lambda spec: schema.DatabaseSpec, ocds: any -> schema.DatabaseStatus {
    """Extract status from observed composed resources."""
    _db_name = spec.region
    _observed_db = ocds.get("${_db_name}-rds", {})
    _observed_status = _observed_db.get("Resource", {}).get("status", {}).get("atProvider", {})

    _endpoint = _observed_status.get("endpoint", "")

    schema.DatabaseStatus {
        provider = "aws"
        region = spec.region
        engine = spec.engine
        endpoint = _endpoint
    }
}
