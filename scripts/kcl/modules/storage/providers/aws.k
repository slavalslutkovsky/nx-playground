"""
AWS S3 resource generation for storage module.
"""

import ...common.types
import ...common.labels
import ..schema

generate = lambda spec: schema.BucketSpec -> [any] {
    """Generate AWS S3 resources from BucketSpec."""
    _bucket_name = spec.name
    _region = spec.region
    _storage_class = schema.map_storage_class("aws", spec.storageClass)

    _label_config = labels.LabelConfig {
        name = _bucket_name
        component = "storage"
        provider = "aws"
        environment = spec.environment
    }
    _tags = labels.generate_tags(_label_config, "aws") | spec.tags

    resources = []

    # Main S3 Bucket
    resources += [{
        apiVersion: "s3.aws.upbound.io/v1beta1"
        kind: "Bucket"
        metadata: {
            name: _bucket_name
            labels: labels.generate_labels(_label_config)
        }
        spec: {
            forProvider: {
                region: _region
                tags: _tags
            }
            deletionPolicy: spec.deletionPolicy
        }
    }]

    # Bucket versioning
    if spec.versioning:
        resources += [{
            apiVersion: "s3.aws.upbound.io/v1beta1"
            kind: "BucketVersioning"
            metadata: {
                name: "${_bucket_name}-versioning"
                labels: labels.generate_labels(_label_config)
            }
            spec: {
                forProvider: {
                    bucketSelector: {
                        matchLabels: {
                            "${labels.LABEL_APP}": _bucket_name
                        }
                    }
                    region: _region
                    versioningConfiguration: [{
                        status: "Enabled"
                    }]
                }
                deletionPolicy: spec.deletionPolicy
            }
        }]

    # Server-side encryption
    if spec.encryption:
        resources += [{
            apiVersion: "s3.aws.upbound.io/v1beta1"
            kind: "BucketServerSideEncryptionConfiguration"
            metadata: {
                name: "${_bucket_name}-encryption"
                labels: labels.generate_labels(_label_config)
            }
            spec: {
                forProvider: {
                    bucketSelector: {
                        matchLabels: {
                            "${labels.LABEL_APP}": _bucket_name
                        }
                    }
                    region: _region
                    rule: [{
                        applyServerSideEncryptionByDefault: [{
                            sseAlgorithm: "AES256"
                        }]
                        bucketKeyEnabled: True
                    }]
                }
                deletionPolicy: spec.deletionPolicy
            }
        }]

    # Public access block (default: block all public access)
    resources += [{
        apiVersion: "s3.aws.upbound.io/v1beta1"
        kind: "BucketPublicAccessBlock"
        metadata: {
            name: "${_bucket_name}-public-access"
            labels: labels.generate_labels(_label_config)
        }
        spec: {
            forProvider: {
                bucketSelector: {
                    matchLabels: {
                        "${labels.LABEL_APP}": _bucket_name
                    }
                }
                region: _region
                blockPublicAcls: not spec.publicAccess
                blockPublicPolicy: not spec.publicAccess
                ignorePublicAcls: not spec.publicAccess
                restrictPublicBuckets: not spec.publicAccess
            }
            deletionPolicy: spec.deletionPolicy
        }
    }]

    # CORS configuration
    if spec.cors:
        resources += [{
            apiVersion: "s3.aws.upbound.io/v1beta1"
            kind: "BucketCorsConfiguration"
            metadata: {
                name: "${_bucket_name}-cors"
                labels: labels.generate_labels(_label_config)
            }
            spec: {
                forProvider: {
                    bucketSelector: {
                        matchLabels: {
                            "${labels.LABEL_APP}": _bucket_name
                        }
                    }
                    region: _region
                    corsRule: [
                        {
                            allowedOrigins: rule.allowedOrigins
                            allowedMethods: rule.allowedMethods
                            allowedHeaders: rule.allowedHeaders
                            maxAgeSeconds: rule.maxAgeSeconds
                        }
                        for rule in spec.cors
                    ]
                }
                deletionPolicy: spec.deletionPolicy
            }
        }]

    # Lifecycle rules
    if spec.lifecycle:
        resources += [{
            apiVersion: "s3.aws.upbound.io/v1beta1"
            kind: "BucketLifecycleConfiguration"
            metadata: {
                name: "${_bucket_name}-lifecycle"
                labels: labels.generate_labels(_label_config)
            }
            spec: {
                forProvider: {
                    bucketSelector: {
                        matchLabels: {
                            "${labels.LABEL_APP}": _bucket_name
                        }
                    }
                    region: _region
                    rule: [
                        _build_lifecycle_rule(rule, spec.deletionPolicy)
                        for rule in spec.lifecycle
                    ]
                }
                deletionPolicy: spec.deletionPolicy
            }
        }]

    resources
}

_build_lifecycle_rule = lambda rule: schema.LifecycleRule, deletionPolicy: str -> any {
    """Build an AWS S3 lifecycle rule."""
    base = {
        id: rule.name
        status: "Enabled" if rule.enabled else "Disabled"
    }

    if rule.prefix:
        base |= {filter: [{prefix: rule.prefix}]}

    if rule.expirationDays > 0:
        base |= {expiration: [{days: rule.expirationDays}]}

    if rule.transitionDays > 0:
        base |= {
            transition: [{
                days: rule.transitionDays
                storageClass: schema.map_storage_class("aws", rule.transitionStorageClass)
            }]
        }

    base
}
