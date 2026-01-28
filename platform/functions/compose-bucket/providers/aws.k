"""
AWS S3 resource generation for bucket composition.
"""

import models.io.upbound.aws.s3.v1beta2 as s3
import models.io.upbound.aws.s3.v1beta1 as s3v1beta1
import schemas.bucket as schema
import schemas.labels as labels
import schemas.helpers as h

generate = lambda spec: schema.BucketSpec, ocds: any -> [any] {
    """Generate AWS S3 resources from BucketSpec."""
    _bucket_name = spec.name
    _region = spec.region
    _storage_class = schema.map_storage_class("aws", spec.storageClass)
    _deletion_policy = spec.deletionPolicy

    _label_config = labels.LabelConfig {
        name = _bucket_name
        component = "storage"
        provider = "aws"
        environment = spec.environment
    }
    _tags = labels.generate_tags(_label_config, "aws") | spec.tags

    resources: [any] = []

    # Main S3 Bucket
    resources += [s3.Bucket {
        metadata = h.metadata(_bucket_name)
        spec = {
            forProvider = {
                region = _region
                tags = _tags
            }
            deletionPolicy = _deletion_policy
        }
    }]

    # Bucket versioning
    if spec.versioning:
        resources += [s3.BucketVersioning {
            metadata = h.metadata("${_bucket_name}-versioning")
            spec = {
                forProvider = {
                    region = _region
                    bucketSelector = {
                        matchControllerRef = True
                    }
                    versioningConfiguration = {
                        status = "Enabled"
                    }
                }
                deletionPolicy = _deletion_policy
            }
        }]

    # Server-side encryption
    if spec.encryption:
        resources += [s3.BucketServerSideEncryptionConfiguration {
            metadata = h.metadata("${_bucket_name}-encryption")
            spec = {
                forProvider = {
                    region = _region
                    bucketSelector = {
                        matchControllerRef = True
                    }
                    $rule = [{
                        applyServerSideEncryptionByDefault = {
                            sseAlgorithm = "AES256"
                        }
                        bucketKeyEnabled = True
                    }]
                }
                deletionPolicy = _deletion_policy
            }
        }]

    # Public access block (default: block all public access)
    resources += [s3v1beta1.BucketPublicAccessBlock {
        metadata = h.metadata("${_bucket_name}-public-access")
        spec = {
            forProvider = {
                region = _region
                bucketSelector = {
                    matchControllerRef = True
                }
                blockPublicAcls = not spec.publicAccess
                blockPublicPolicy = not spec.publicAccess
                ignorePublicAcls = not spec.publicAccess
                restrictPublicBuckets = not spec.publicAccess
            }
            deletionPolicy = _deletion_policy
        }
    }]

    # CORS configuration
    if spec.cors:
        resources += [{
            apiVersion = "s3.aws.upbound.io/v1beta1"
            kind = "BucketCorsConfiguration"
            metadata = h.metadata("${_bucket_name}-cors")
            spec = {
                forProvider = {
                    bucketSelector = {
                        matchControllerRef = True
                    }
                    region = _region
                    corsRule = [
                        {
                            allowedOrigins = rule.allowedOrigins
                            allowedMethods = rule.allowedMethods
                            allowedHeaders = rule.allowedHeaders
                            maxAgeSeconds = rule.maxAgeSeconds
                        }
                        for rule in spec.cors
                    ]
                }
                deletionPolicy = _deletion_policy
            }
        }]

    # Lifecycle rules
    if spec.lifecycle:
        resources += [{
            apiVersion = "s3.aws.upbound.io/v1beta1"
            kind = "BucketLifecycleConfiguration"
            metadata = h.metadata("${_bucket_name}-lifecycle")
            spec = {
                forProvider = {
                    bucketSelector = {
                        matchControllerRef = True
                    }
                    region = _region
                    $rule = [
                        _build_lifecycle_rule(rule)
                        for rule in spec.lifecycle
                    ]
                }
                deletionPolicy = _deletion_policy
            }
        }]

    resources
}

_build_lifecycle_rule = lambda rule: schema.LifecycleRule -> any {
    """Build an AWS S3 lifecycle rule."""
    base: {str:any} = {
        id = rule.name
        status = "Enabled" if rule.enabled else "Disabled"
    }

    if rule.prefix:
        base |= {$filter = [{prefix = rule.prefix}]}

    if rule.expirationDays > 0:
        base |= {$expiration = [{days = rule.expirationDays}]}

    if rule.transitionDays > 0:
        base |= {
            $transition = [{
                days = rule.transitionDays
                storageClass = schema.map_storage_class("aws", rule.transitionStorageClass)
            }]
        }

    base
}

get_status = lambda spec: schema.BucketSpec, ocds: any -> schema.BucketStatus {
    """Extract status from observed composed resources."""
    _observed_bucket = ocds.get(spec.name, {})
    _observed_status = _observed_bucket.get("Resource", {}).get("status", {}).get("atProvider", {})

    _bucket_id = _observed_status.get("id", spec.name)
    _bucket_domain = _observed_status.get("bucketRegionalDomainName", "")

    schema.BucketStatus {
        provider = "aws"
        region = spec.region
        bucketName = _bucket_id
        url = "s3://${_bucket_id}"
        id = _bucket_id
        cloudUrl = "https://${_bucket_domain}" if _bucket_domain else "https://${spec.name}.s3.${spec.region}.amazonaws.com"
    }
}
