"""
Unified Storage Bucket Function for Multi-Cloud Management

This module provides a single entry point for creating storage buckets across
AWS S3, GCP Cloud Storage, and Azure Blob Storage. Provider selection is
handled via labels on the composite resource.

Labels:
  - cloud.provider/aws: "true"    -> Creates AWS S3 bucket
  - cloud.provider/gcp: "true"    -> Creates GCP Storage bucket
  - cloud.provider/azure: "true"  -> Creates Azure Storage container
"""

# Import typed schemas from generated Upbound models
import models.io.upbound.awsm.s3.v1beta1 as s3
import models.io.upbound.gcpm.storage.v1beta1 as gcpstorage
import models.io.upbound.azure.storage.v1beta1 as azurestoragev1
import models.io.upbound.azure.storage.v1beta2 as azurestoragev2
import models.k8s.apimachinery.pkg.apis.meta.v1 as v1

# Provider label constants
LABEL_AWS = "cloud.provider/aws"
LABEL_GCP = "cloud.provider/gcp"
LABEL_AZURE = "cloud.provider/azure"

# Schema for unified storage parameters
schema StorageParams:
    """Common parameters for all cloud storage providers"""
    name: str
    location: str
    acl?: str = "private"
    versioning?: bool = False
    tags?: {str: str} = {}
    # Provider-specific configs
    aws?: AWSConfig
    gcp?: GCPConfig
    azure?: AzureConfig

    check:
        len(name) >= 3 and len(name) <= 63, "Bucket name must be 3-63 characters"
        acl in ["private", "public-read", "authenticated-read"], "Invalid ACL value"

schema AWSConfig:
    """AWS S3 specific configuration"""
    region?: str
    objectLockEnabled?: bool = False
    accelerationStatus?: str

schema GCPConfig:
    """GCP Cloud Storage specific configuration"""
    project?: str
    storageClass?: str = "STANDARD"
    uniformBucketLevelAccess?: bool = True

schema AzureConfig:
    """Azure Blob Storage specific configuration"""
    resourceGroup: str
    accountTier?: str = "Standard"
    accountReplicationType?: str = "LRS"
    containerAccessType?: str = "private"

# Main composition function
createStorageBucket = lambda oxr: any, ocds: any, params: StorageParams -> [any] {
    """
    Creates storage bucket(s) based on provider labels.
    """
    labels = oxr?.metadata?.labels or {}

    # Determine which providers to create resources for
    _is_aws = labels.get(LABEL_AWS, "false") == "true"
    _is_gcp = labels.get(LABEL_GCP, "false") == "true"
    _is_azure = labels.get(LABEL_AZURE, "false") == "true"

    # Build resources based on labels using list comprehension
    _aws_resources = createAWSResources(params) if _is_aws else []
    _gcp_resources = createGCPResources(params) if _is_gcp else []
    _azure_resources = createAzureResources(params) if _is_azure else []

    _aws_resources + _gcp_resources + _azure_resources
}

# AWS S3 Resource Creation using typed schemas
createAWSResources = lambda params: StorageParams -> [any] {
    """Creates AWS S3 bucket and related resources using typed KCL schemas"""
    _bucket_name = "${params.name}-aws"
    _region = params.aws?.region or params.location
    _merged_tags = {k: v for k, v in params.tags} | {"ManagedBy": "crossplane", "Provider": "aws"}

    # Typed S3 Bucket using the schema
    _bucket = s3.Bucket {
        metadata = v1.ObjectMeta {
            name = _bucket_name
            labels = {
                "${LABEL_AWS}" = "true"
                "storage.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                region = _region
                tags = _merged_tags
                objectLockEnabled = params.aws?.objectLockEnabled
            }
        }
    }

    # Typed BucketACL using the schema
    _acl = s3.BucketACL {
        metadata = v1.ObjectMeta {
            name = "${_bucket_name}-acl"
        }
        spec = {
            forProvider = {
                bucketRef = {
                    name = _bucket_name
                }
                region = _region
                acl = params.acl
            }
        }
    }

    # Typed BucketVersioning (conditional) using the schema
    _versioning = s3.BucketVersioning {
        metadata = v1.ObjectMeta {
            name = "${_bucket_name}-versioning"
        }
        spec = {
            forProvider = {
                bucketRef = {
                    name = _bucket_name
                }
                region = _region
                versioningConfiguration = {
                    status = "Enabled"
                }
            }
        }
    } if params.versioning else None

    [r for r in [_bucket, _acl, _versioning] if r]
}

# GCP Storage Resource Creation using typed schemas
createGCPResources = lambda params: StorageParams -> [any] {
    """Creates GCP Cloud Storage bucket and related resources using typed KCL schemas"""
    _bucket_name = "${params.name}-gcp"
    _merged_labels = {k: v for k, v in params.tags} | {"managed-by": "crossplane", "provider": "gcp"}
    _versioning_config = {enabled = True} if params.versioning else None

    [
        gcpstorage.Bucket {
            metadata = v1.ObjectMeta {
                name = _bucket_name
                labels = {
                    "${LABEL_GCP}" = "true"
                    "storage.crossplane.io/name" = params.name
                }
            }
            spec = {
                forProvider = {
                    location = params.location
                    storageClass = params.gcp?.storageClass or "STANDARD"
                    uniformBucketLevelAccess = params.gcp?.uniformBucketLevelAccess or True
                    versioning = _versioning_config
                    labels = _merged_labels
                    project = params.gcp?.project
                }
            }
        }
    ]
}

# Azure Storage Resource Creation using typed schemas
createAzureResources = lambda params: StorageParams -> [any] {
    """Creates Azure Storage Account and Container using typed KCL schemas"""
    _storage_name = params.name.replace("-", "").replace("_", "")[:24]
    _rg = params.azure?.resourceGroup or "default-rg"
    _merged_tags = {k: v for k, v in params.tags} | {"ManagedBy": "crossplane", "Provider": "azure"}

    [
        # Typed Storage Account using the schema
        azurestoragev2.Account {
            metadata = v1.ObjectMeta {
                name = "${_storage_name}sa"
                labels = {
                    "${LABEL_AZURE}" = "true"
                    "storage.crossplane.io/name" = params.name
                }
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    location = params.location
                    accountTier = params.azure?.accountTier or "Standard"
                    accountReplicationType = params.azure?.accountReplicationType or "LRS"
                    blobProperties = {
                        versioningEnabled = params.versioning
                    } if params.versioning else None
                    tags = _merged_tags
                }
            }
        }
        # Typed Blob Container using the schema
        azurestoragev1.Container {
            metadata = v1.ObjectMeta {
                name = "${_storage_name}-container"
                labels = {
                    "${LABEL_AZURE}" = "true"
                    "storage.crossplane.io/name" = params.name
                }
            }
            spec = {
                forProvider = {
                    storageAccountNameRef = {
                        name = "${_storage_name}sa"
                    }
                    containerAccessType = params.azure?.containerAccessType or "private"
                }
            }
        }
    ]
}

# Utility: Filter resources by provider label
filterByProvider = lambda resources: [any], provider: str -> [any] {
    """Filters composed resources by provider label"""
    _label_key = provider if provider.startswith("cloud.provider/") else "cloud.provider/${provider}"
    [r for r in resources if r?.metadata?.labels?.get(_label_key, "false") == "true"]
}

# Utility: Get all providers from labels
getActiveProviders = lambda labels: {str: str} -> [str] {
    """Returns list of active providers based on labels"""
    _aws = ["aws"] if (labels[LABEL_AWS] or "false") == "true" else []
    _gcp = ["gcp"] if (labels[LABEL_GCP] or "false") == "true" else []
    _azure = ["azure"] if (labels[LABEL_AZURE] or "false") == "true" else []
    _aws + _gcp + _azure
}
