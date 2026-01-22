"""
Multi-provider storage composition.
Creates AWS S3 or GCP GCS bucket based on provider field.
"""

import models.io.upbound.aws.s3.v1beta2 as s3
import models.io.upbound.aws.s3.v1beta1 as s3v1beta1
import models.io.upbound.gcp.storage.v1beta2 as gcs
import schemas.bucket as schema
import schemas.helpers as h
import schemas.mappings as m

# Crossplane function inputs
oxr = option("params").oxr  # observed composite resource
_ocds = option("params").ocds  # observed composed resources
dxr = option("params").dxr  # desired composite resource
dcds = option("params").dcds  # desired composed resources

# Type-annotated spec extraction (IDE autocomplete enabled)
_raw_spec = oxr.spec
_spec: schema.BucketSpec = schema.BucketSpec {
    provider = _raw_spec.provider
    name = _raw_spec.name
    region = _raw_spec.region
    environment = _raw_spec.environment or "dev"
    storageClass = _raw_spec.storageClass or "standard"
    versioning = _raw_spec.versioning or False
    encryption = _raw_spec.encryption or True
    publicAccess = _raw_spec.publicAccess or False
    deletionPolicy = _raw_spec.deletionPolicy or "Delete"
}

# Use typed spec
_name = _spec.name
_provider = _spec.provider
_region = _spec.region
_env = _spec.environment
_storage_class = _spec.storageClass
_versioning = _spec.versioning
_encryption = _spec.encryption
_deletion_policy = _spec.deletionPolicy

# AWS S3 Bucket
_aws_bucket = s3.Bucket {
    metadata = h.metadata(_name)
    spec = {
        forProvider = {
            region = _region
            tags = h.aws_tags(_env)
        }
        deletionPolicy = _deletion_policy
    }
}

# AWS S3 BucketVersioning (using KCL schema)
_aws_bucket_versioning = s3.BucketVersioning {
    metadata = h.metadata("${_name}-versioning")
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
} if _versioning else None

# AWS S3 BucketServerSideEncryptionConfiguration (using KCL schema)
_aws_bucket_encryption = s3.BucketServerSideEncryptionConfiguration {
    metadata = h.metadata("${_name}-encryption")
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
} if _encryption else None

# AWS S3 BucketPublicAccessBlock (using KCL schema)
# Note: Uses v1beta1 as this resource is not available in v1beta2
_aws_bucket_public_access_block = s3v1beta1.BucketPublicAccessBlock {
    metadata = h.metadata("${_name}-public-access")
    spec = {
        forProvider = {
            region = _region
            bucketSelector = {
                matchControllerRef = True
            }
            # Block public access by default (inverted logic: publicAccess=false means block=true)
            blockPublicAcls = not _spec.publicAccess
            blockPublicPolicy = not _spec.publicAccess
            ignorePublicAcls = not _spec.publicAccess
            restrictPublicBuckets = not _spec.publicAccess
        }
        deletionPolicy = _deletion_policy
    }
}

# GCP GCS Bucket
_gcp_bucket = gcs.Bucket {
    metadata = h.metadata(_name)
    spec = {
        forProvider = {
            location = _region
            storageClass = m.get_gcp_storage_class(_storage_class)
            uniformBucketLevelAccess = True
            versioning = {
                enabled = _versioning
            } if _versioning else None
            labels = h.gcp_labels(_env)
        }
        deletionPolicy = _deletion_policy
    }
}

# Select bucket based on provider
_bucket = _aws_bucket if _provider == "aws" else _gcp_bucket

# Get observed bucket status from ocds (if exists)
# The bucket resource name matches _name from metadata annotation
_observed_bucket = _ocds.get(_name, {})
_observed_status = _observed_bucket.get("Resource", {}).get("status", {}).get("atProvider", {})

# Extract status fields from observed managed resource
# AWS S3 Bucket status fields: id, arn, bucketDomainName, bucketRegionalDomainName, region
# GCP GCS Bucket status fields: id, selfLink, url
_bucket_id = _observed_status.get("id", _name)
_bucket_arn = _observed_status.get("arn", "")

# Generate URLs - prefer observed values, fallback to constructed
_aws_url = "s3://${_bucket_id}"
_aws_cloud_url = _observed_status.get("bucketRegionalDomainName", "") or "https://${_name}.s3.${_region}.amazonaws.com"
_gcp_url = _observed_status.get("url", "") or "gs://${_name}"
_gcp_cloud_url = _observed_status.get("selfLink", "") or "https://storage.googleapis.com/${_name}"

# Update composite resource status
# Using dict for hyphenated keys (cloud-url) that match jsonPath in additionalPrinterColumns
_status = {
    provider = _provider
    region = _region
    bucketName = _bucket_id
    url = _aws_url if _provider == "aws" else _gcp_url
    id = _bucket_id
    "cloud-url" = ("https://" + _aws_cloud_url if _aws_cloud_url and not _aws_cloud_url.startswith("https://") else _aws_cloud_url) if _provider == "aws" else _gcp_cloud_url
}

_dxr = {
    **oxr
    status = _status
}

# Collect AWS-specific resources (only for AWS provider)
# Filter out None values (resources disabled by spec flags like versioning=false)
_aws_extra_resources = [
    r for r in [
        _aws_bucket_versioning,
        _aws_bucket_encryption,
        _aws_bucket_public_access_block
    ] if r != None
] if _provider == "aws" else []

# Output items
items = [_bucket] + _aws_extra_resources + [_dxr]
