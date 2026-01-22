"""
Tests for storage module providers.
Run with: kcl test scripts/kcl/modules/storage/
"""

import ..schema
import .aws
import .gcp

# Test AWS S3 generation - basic
test_aws_generate_basic = lambda {
    spec = schema.BucketSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-bucket"
    }
    resources = aws.generate(spec)

    # Should have bucket + public access block (encryption default)
    assert len(resources) >= 2

    # Check main bucket
    bucket = [r for r in resources if r.kind == "Bucket"][0]
    assert bucket.metadata.name == "test-bucket"
    assert bucket.spec.forProvider.region == "us-east-1"
}

# Test AWS S3 generation - with versioning
test_aws_generate_versioning = lambda {
    spec = schema.BucketSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-bucket"
        versioning = True
    }
    resources = aws.generate(spec)

    versioning = [r for r in resources if r.kind == "BucketVersioning"]
    assert len(versioning) == 1
    assert versioning[0].spec.forProvider.versioningConfiguration[0].status == "Enabled"
}

# Test AWS S3 generation - with encryption
test_aws_generate_encryption = lambda {
    spec = schema.BucketSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-bucket"
        encryption = True
    }
    resources = aws.generate(spec)

    encryption = [r for r in resources if r.kind == "BucketServerSideEncryptionConfiguration"]
    assert len(encryption) == 1
    assert encryption[0].spec.forProvider.rule[0].applyServerSideEncryptionByDefault[0].sseAlgorithm == "AES256"
}

# Test AWS S3 generation - with CORS
test_aws_generate_cors = lambda {
    spec = schema.BucketSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-bucket"
        cors = [{
            allowedOrigins = ["https://example.com"]
            allowedMethods = ["GET", "POST"]
        }]
    }
    resources = aws.generate(spec)

    cors = [r for r in resources if r.kind == "BucketCorsConfiguration"]
    assert len(cors) == 1
    assert cors[0].spec.forProvider.corsRule[0].allowedOrigins[0] == "https://example.com"
}

# Test AWS S3 generation - with lifecycle
test_aws_generate_lifecycle = lambda {
    spec = schema.BucketSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-bucket"
        lifecycle = [{
            name = "expire-logs"
            prefix = "logs/"
            expirationDays = 30
        }]
    }
    resources = aws.generate(spec)

    lifecycle = [r for r in resources if r.kind == "BucketLifecycleConfiguration"]
    assert len(lifecycle) == 1
    assert lifecycle[0].spec.forProvider.rule[0].id == "expire-logs"
}

# Test GCP GCS generation - basic
test_gcp_generate_basic = lambda {
    spec = schema.BucketSpec {
        provider = "gcp"
        region = "us-east-1"
        name = "test-bucket"
    }
    resources = gcp.generate(spec)

    assert len(resources) >= 1

    bucket = [r for r in resources if r.kind == "Bucket"][0]
    assert bucket.metadata.name == "test-bucket"
    # Region should be mapped from AWS to GCP format
    assert bucket.spec.forProvider.location == "US-EAST1"
    assert bucket.spec.forProvider.uniformBucketLevelAccess == True
}

# Test GCP GCS generation - with versioning
test_gcp_generate_versioning = lambda {
    spec = schema.BucketSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-bucket"
        versioning = True
    }
    resources = gcp.generate(spec)

    bucket = [r for r in resources if r.kind == "Bucket"][0]
    assert bucket.spec.forProvider.versioning[0].enabled == True
}

# Test GCP GCS generation - storage class mapping
test_gcp_storage_class = lambda {
    spec = schema.BucketSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-bucket"
        storageClass = "nearline"
    }
    resources = gcp.generate(spec)

    bucket = [r for r in resources if r.kind == "Bucket"][0]
    assert bucket.spec.forProvider.storageClass == "NEARLINE"
}

# Test GCP GCS generation - public access creates IAM binding
test_gcp_public_access = lambda {
    spec = schema.BucketSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-bucket"
        publicAccess = True
        encryption = True  # Required for public buckets
    }
    resources = gcp.generate(spec)

    iam = [r for r in resources if r.kind == "BucketIAMMember"]
    assert len(iam) == 1
    assert iam[0].spec.forProvider.member == "allUsers"
}

# Test GCP GCS generation - with lifecycle
test_gcp_generate_lifecycle = lambda {
    spec = schema.BucketSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-bucket"
        lifecycle = [{
            name = "archive-old"
            transitionDays = 90
            transitionStorageClass = "coldline"
        }]
    }
    resources = gcp.generate(spec)

    bucket = [r for r in resources if r.kind == "Bucket"][0]
    assert bucket.spec.forProvider.lifecycleRule[0].action.storageClass == "COLDLINE"
    assert bucket.spec.forProvider.lifecycleRule[0].condition.age == 90
}
