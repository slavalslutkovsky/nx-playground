"""
GCP Cloud Storage resource generation for storage module.
"""

import ...common.types
import ...common.labels
import ..schema

generate = lambda spec: schema.BucketSpec -> [any] {
    """Generate GCP Cloud Storage resources from BucketSpec."""
    _bucket_name = spec.name
    _location = types.map_region("gcp", spec.region).upper()
    _storage_class = schema.map_storage_class("gcp", spec.storageClass)

    _label_config = labels.LabelConfig {
        name = _bucket_name
        component = "storage"
        provider = "gcp"
        environment = spec.environment
    }
    _labels = labels.generate_tags(_label_config, "gcp") | {
        k.lower().replace("-", "_"): v.lower()
        for k, v in spec.tags
        if len(k) <= 63 and len(v) <= 63
    }

    resources = []

    # Main GCS Bucket
    bucket = {
        apiVersion: "storage.gcp.upbound.io/v1beta1"
        kind: "Bucket"
        metadata: {
            name: _bucket_name
            labels: labels.generate_labels(_label_config)
        }
        spec: {
            forProvider: {
                location: _location
                storageClass: _storage_class
                labels: _labels
                uniformBucketLevelAccess: True
                publicAccessPrevention: "enforced" if not spec.publicAccess else "inherited"
            }
            deletionPolicy: spec.deletionPolicy
        }
    }

    # Add versioning if enabled
    if spec.versioning:
        bucket.spec.forProvider |= {
            versioning: [{
                enabled: True
            }]
        }

    # Add encryption if enabled (GCP uses default encryption, but we can specify CMEK)
    if spec.encryption:
        bucket.spec.forProvider |= {
            encryption: [{
                defaultKmsKeyName: ""  # Uses Google-managed keys by default
            }]
        }

    # Add CORS rules
    if spec.cors:
        bucket.spec.forProvider |= {
            cors: [
                {
                    origin: rule.allowedOrigins
                    method: rule.allowedMethods
                    responseHeader: rule.allowedHeaders
                    maxAgeSeconds: rule.maxAgeSeconds
                }
                for rule in spec.cors
            ]
        }

    # Add lifecycle rules
    if spec.lifecycle:
        bucket.spec.forProvider |= {
            lifecycleRule: [
                _build_lifecycle_rule(rule)
                for rule in spec.lifecycle
            ]
        }

    resources += [bucket]

    # IAM binding for public access if enabled
    if spec.publicAccess:
        resources += [{
            apiVersion: "storage.gcp.upbound.io/v1beta1"
            kind: "BucketIAMMember"
            metadata: {
                name: "${_bucket_name}-public-read"
                labels: labels.generate_labels(_label_config)
            }
            spec: {
                forProvider: {
                    bucketSelector: {
                        matchLabels: {
                            "${labels.LABEL_APP}": _bucket_name
                        }
                    }
                    role: "roles/storage.objectViewer"
                    member: "allUsers"
                }
                deletionPolicy: spec.deletionPolicy
            }
        }]

    resources
}

_build_lifecycle_rule = lambda rule: schema.LifecycleRule -> any {
    """Build a GCP lifecycle rule."""
    result = {
        action: {}
        condition: {}
    }

    if rule.expirationDays > 0:
        result.action |= {type: "Delete"}
        result.condition |= {age: rule.expirationDays}
    elif rule.transitionDays > 0:
        result.action |= {
            type: "SetStorageClass"
            storageClass: schema.map_storage_class("gcp", rule.transitionStorageClass)
        }
        result.condition |= {age: rule.transitionDays}

    if rule.prefix:
        result.condition |= {matchesPrefix: [rule.prefix]}

    result
}
