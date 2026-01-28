"""
Unit tests for bucket bkt.
"""

import .bucket as bkt
import .common as common

# ============================================================================
# Schema Validation Tests
# ============================================================================

test_bucket_spec_basic = lambda {
    """Test basic bucket spec creation."""
    spec = bkt.BucketSpec {
        provider = "aws"
        name = "test-bucket"
        region = "us-east-1"
    }
    assert spec.name == "test-bucket"
    assert spec.provider == "aws"
    assert spec.region == "us-east-1"
    assert spec.storageClass == "standard"
    assert spec.versioning == False
    assert spec.encryption == True
    assert spec.publicAccess == False
    assert spec.deletionPolicy == "Delete"
    assert spec.environment == "dev"
}

test_bucket_spec_with_options = lambda {
    """Test bucket spec with all options."""
    spec = bkt.BucketSpec {
        provider = "gcp"
        name = "my-bucket"
        region = "europe-west1"
        storageClass = "nearline"
        versioning = True
        encryption = True
        publicAccess = False
        environment = "prod"
        deletionPolicy = "Orphan"
    }
    assert spec.storageClass == "nearline"
    assert spec.versioning == True
    assert spec.environment == "prod"
    assert spec.deletionPolicy == "Orphan"
}

test_bucket_spec_with_cors = lambda {
    """Test bucket spec with CORS configuration."""
    spec = bkt.BucketSpec {
        provider = "aws"
        name = "cors-bucket"
        region = "us-east-1"
        cors = [
            bkt.CorsRule {
                allowedOrigins = ["https://example.com"]
                allowedMethods = ["GET", "PUT"]
                allowedHeaders = ["*"]
                maxAgeSeconds = 7200
            }
        ]
    }
    assert len(spec.cors) == 1
    assert spec.cors[0].allowedOrigins[0] == "https://example.com"
    assert spec.cors[0].maxAgeSeconds == 7200
}

test_bucket_spec_with_lifecycle = lambda {
    """Test bucket spec with lifecycle rules."""
    spec = bkt.BucketSpec {
        provider = "gcp"
        name = "lifecycle-bucket"
        region = "us-east-1"
        lifecycle = [
            bkt.LifecycleRule {
                name = "archive-old"
                prefix = "logs/"
                enabled = True
                transitionDays = 90
                transitionStorageClass = "coldline"
            }
        ]
    }
    assert len(spec.lifecycle) == 1
    assert spec.lifecycle[0].name == "archive-old"
    assert spec.lifecycle[0].transitionDays == 90
}

# ============================================================================
# Storage Class Mapping Tests
# ============================================================================

test_storage_class_map_aws = lambda {
    """Test AWS storage class mapping."""
    assert bkt.map_storage_class("aws", "standard") == "STANDARD"
    assert bkt.map_storage_class("aws", "nearline") == "STANDARD_IA"
    assert bkt.map_storage_class("aws", "coldline") == "GLACIER"
    assert bkt.map_storage_class("aws", "archive") == "DEEP_ARCHIVE"
}

test_storage_class_map_gcp = lambda {
    """Test GCP storage class mapping."""
    assert bkt.map_storage_class("gcp", "standard") == "STANDARD"
    assert bkt.map_storage_class("gcp", "nearline") == "NEARLINE"
    assert bkt.map_storage_class("gcp", "coldline") == "COLDLINE"
    assert bkt.map_storage_class("gcp", "archive") == "ARCHIVE"
}

# ============================================================================
# Tags Field Tests
# ============================================================================

test_bucket_spec_with_tags = lambda {
    """Test bucket spec with custom tags."""
    spec = bkt.BucketSpec {
        provider = "aws"
        name = "tagged-bucket"
        region = "us-east-1"
        tags = {
            "project" = "my-project"
            "team" = "platform"
        }
    }
    assert len(spec.tags) == 2
    assert spec.tags["project"] == "my-project"
    assert spec.tags["team"] == "platform"
}
