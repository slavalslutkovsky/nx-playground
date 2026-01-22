"""
Tests for storage module schema.
Run with: kcl test scripts/kcl/modules/storage/
"""

import .schema

# Test BucketSpec defaults
test_bucket_spec_defaults = lambda {
    spec = schema.BucketSpec {
        provider = "aws"
        region = "us-east-1"
        name = "my-bucket"
    }
    assert spec.storageClass == "standard"
    assert spec.versioning == False
    assert spec.encryption == True
    assert spec.publicAccess == False
    assert spec.cors == []
    assert spec.lifecycle == []
}

# Test BucketSpec with all options
test_bucket_spec_full = lambda {
    spec = schema.BucketSpec {
        provider = "gcp"
        region = "us-central1"
        environment = "prod"
        name = "prod-data-bucket"
        storageClass = "nearline"
        versioning = True
        encryption = True
        publicAccess = False
        tags = {
            team: "data"
        }
    }
    assert spec.provider == "gcp"
    assert spec.storageClass == "nearline"
    assert spec.versioning == True
}

# Test storage class mapping
test_storage_class_map_aws = lambda {
    result = schema.map_storage_class("aws", "standard")
    assert result == "STANDARD"
}

test_storage_class_map_aws_coldline = lambda {
    result = schema.map_storage_class("aws", "coldline")
    assert result == "GLACIER"
}

test_storage_class_map_gcp = lambda {
    result = schema.map_storage_class("gcp", "standard")
    assert result == "STANDARD"
}

test_storage_class_map_gcp_nearline = lambda {
    result = schema.map_storage_class("gcp", "nearline")
    assert result == "NEARLINE"
}

# Test CorsRule
test_cors_rule = lambda {
    cors_rule = schema.CorsRule {
        allowedOrigins = ["https://example.com"]
        allowedMethods = ["GET", "POST"]
        maxAgeSeconds = 7200
    }
    assert cors_rule.allowedOrigins[0] == "https://example.com"
    assert len(cors_rule.allowedMethods) == 2
    assert cors_rule.maxAgeSeconds == 7200
}

# Test LifecycleRule
test_lifecycle_rule_expiration = lambda {
    lc_rule = schema.LifecycleRule {
        name = "delete-old"
        prefix = "logs/"
        expirationDays = 30
    }
    assert lc_rule.name == "delete-old"
    assert lc_rule.prefix == "logs/"
    assert lc_rule.expirationDays == 30
}

test_lifecycle_rule_transition = lambda {
    lc_rule = schema.LifecycleRule {
        name = "archive-old"
        transitionDays = 90
        transitionStorageClass = "coldline"
    }
    assert lc_rule.transitionDays == 90
    assert lc_rule.transitionStorageClass == "coldline"
}
