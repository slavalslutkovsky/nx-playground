"""
GCP Cloud Storage resource generation for bucket composition.
"""

import models.io.upbound.gcp.storage.v1beta2 as gcs
import schemas.bucket as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.common as c

generate = lambda spec: schema.BucketSpec, ocds: any -> [any] {
    """Generate GCP Cloud Storage resources from BucketSpec."""
    _bucket_name = spec.name
    _location = c.map_region("gcp", spec.region).upper()
    _storage_class = schema.map_storage_class("gcp", spec.storageClass)
    _deletion_policy = spec.deletionPolicy

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

    resources: [any] = []

    # Build bucket forProvider config
    _for_provider: {str:any} = {
        location = _location
        storageClass = _storage_class
        labels = _labels
        uniformBucketLevelAccess = True
        publicAccessPrevention = "enforced" if not spec.publicAccess else "inherited"
    }

    # Add versioning if enabled
    if spec.versioning:
        _for_provider |= {
            versioning = [{
                enabled = True
            }]
        }

    # Add CORS rules if specified
    if spec.cors:
        _for_provider |= {
            cors = [
                {
                    origin = rule.allowedOrigins
                    method = rule.allowedMethods
                    responseHeader = rule.allowedHeaders
                    maxAgeSeconds = rule.maxAgeSeconds
                }
                for rule in spec.cors
            ]
        }

    # Add lifecycle rules if specified
    if spec.lifecycle:
        _for_provider |= {
            lifecycleRule = [
                _build_lifecycle_rule(rule)
                for rule in spec.lifecycle
            ]
        }

    # Main GCS Bucket
    resources += [gcs.Bucket {
        metadata = h.metadata(_bucket_name)
        spec = {
            forProvider = _for_provider
            deletionPolicy = _deletion_policy
        }
    }]

    # IAM binding for public access if enabled
    if spec.publicAccess:
        resources += [{
            apiVersion = "storage.gcp.upbound.io/v1beta1"
            kind = "BucketIAMMember"
            metadata = h.metadata("${_bucket_name}-public-read")
            spec = {
                forProvider = {
                    bucketSelector = {
                        matchControllerRef = True
                    }
                    role = "roles/storage.objectViewer"
                    member = "allUsers"
                }
                deletionPolicy = _deletion_policy
            }
        }]

    resources
}

_build_lifecycle_rule = lambda rule: schema.LifecycleRule -> any {
    """Build a GCP lifecycle rule."""
    result: {str:any} = {
        action = {}
        condition = {}
    }

    if rule.expirationDays > 0:
        result.action |= {$type = "Delete"}
        result.condition |= {age = rule.expirationDays}
    elif rule.transitionDays > 0:
        result.action |= {
            $type = "SetStorageClass"
            storageClass = schema.map_storage_class("gcp", rule.transitionStorageClass)
        }
        result.condition |= {age = rule.transitionDays}

    if rule.prefix:
        result.condition |= {matchesPrefix = [rule.prefix]}

    result
}

get_status = lambda spec: schema.BucketSpec, ocds: any -> schema.BucketStatus {
    """Extract status from observed composed resources."""
    _observed_bucket = ocds.get(spec.name, {})
    _observed_status = _observed_bucket.get("Resource", {}).get("status", {}).get("atProvider", {})

    _bucket_url = _observed_status.get("url", "")
    _bucket_self_link = _observed_status.get("selfLink", "")

    schema.BucketStatus {
        provider = "gcp"
        region = spec.region
        bucketName = spec.name
        url = _bucket_url if _bucket_url else "gs://${spec.name}"
        id = spec.name
        cloudUrl = _bucket_self_link if _bucket_self_link else "https://storage.googleapis.com/${spec.name}"
    }
}
