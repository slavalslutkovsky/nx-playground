"""
Composition generator for storage module.
Run: kcl run scripts/kcl/modules/storage/render/composition.k
"""

# KCL source that runs inside function-kcl (as a string)
_kcl_source = """\
oxr = option("params").oxr
spec = oxr.spec

_name = spec.name
_provider = spec.provider
_region = spec.region
_env = spec.get("environment", "dev")
_tags = spec.get("tags", {})

# AWS S3 Bucket
_aws_bucket = {
    apiVersion: "s3.aws.upbound.io/v1beta1"
    kind: "Bucket"
    metadata.name: _name
    spec: {
        forProvider: {
            region: _region
            tags: _tags | {
                "managed-by": "crossplane"
                environment: _env
            }
        }
        deletionPolicy: spec.get("deletionPolicy", "Delete")
    }
}

# GCP GCS Bucket
_gcp_bucket = {
    apiVersion: "storage.gcp.upbound.io/v1beta1"
    kind: "Bucket"
    metadata.name: _name
    spec: {
        forProvider: {
            location: _region
            storageClass: "STANDARD"
            uniformBucketLevelAccess: True
            labels: {
                "managed-by": "crossplane"
                environment: _env
            }
        }
        deletionPolicy: spec.get("deletionPolicy", "Delete")
    }
}

_bucket = _aws_bucket if _provider == "aws" else _gcp_bucket

dxr = {
    **oxr
    status: {
        provider: _provider
        region: _region
        bucketName: _name
    }
}

items = [_bucket, dxr]
"""

# Output the Composition manifest
[{
    apiVersion: "apiextensions.crossplane.io/v1"
    kind: "Composition"
    metadata: {
        name: "xbucket"
        labels: {
            "crossplane.io/module": "storage"
        }
    }
    spec: {
        compositeTypeRef: {
            apiVersion: "storage.platform.io/v1alpha1"
            kind: "XBucket"
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
}]
