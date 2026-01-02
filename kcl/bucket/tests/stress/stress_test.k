"""
Stress test for the unified storage function.
Generates a large number of resources to test performance and memory usage.
"""

import functions.storage.main as storage

# Configuration - can be overridden via -D flag
BUCKET_COUNT = option("BUCKET_COUNT") or 100
PROVIDERS = ["aws", "gcp", "azure"]

# Generate test parameters for multiple buckets
generateBuckets = lambda count: int, provider: str -> [storage.StorageParams] {
    _location = "us-east-1" if provider == "aws" else ("US" if provider == "gcp" else "eastus")

    [
        storage.StorageParams {
            name = "stress-test-bucket-${provider}-${i}"
            location = _location
            acl = "private"
            versioning = i % 2 == 0  # Alternate versioning
            tags = {
                "test" = "stress"
                "index" = "${i}"
                "provider" = provider
            }
            aws = storage.AWSConfig {
                region = "us-east-1"
            } if provider == "aws" else None
            gcp = storage.GCPConfig {
                storageClass = "STANDARD"
            } if provider == "gcp" else None
            azure = storage.AzureConfig {
                resourceGroup = "stress-test-rg"
                accountTier = "Standard"
                accountReplicationType = "LRS"
            } if provider == "azure" else None
        }
        for i in range(int(count))
    ]
}

# Generate resources for each provider
_aws_params = generateBuckets(BUCKET_COUNT, "aws")
_gcp_params = generateBuckets(BUCKET_COUNT, "gcp")
_azure_params = generateBuckets(BUCKET_COUNT, "azure")

# Create all resources
_all_aws_resources = [r for params in _aws_params for r in storage.createAWSResources(params)]
_all_gcp_resources = [r for params in _gcp_params for r in storage.createGCPResources(params)]
_all_azure_resources = [r for params in _azure_params for r in storage.createAzureResources(params)]

# Combine all resources
_total_resources = _all_aws_resources + _all_gcp_resources + _all_azure_resources

# Output statistics
stress_test_results = {
    aws_resource_count = len(_all_aws_resources)
    gcp_resource_count = len(_all_gcp_resources)
    azure_resource_count = len(_all_azure_resources)
    total_resources = len(_total_resources)
    buckets_per_provider = BUCKET_COUNT
    providers_tested = len(PROVIDERS)
}

# Assertions
assert len(_all_aws_resources) >= int(BUCKET_COUNT), "AWS should have at least ${BUCKET_COUNT} resources"
assert len(_all_gcp_resources) == int(BUCKET_COUNT), "GCP should have ${BUCKET_COUNT} resources"
assert len(_all_azure_resources) == int(BUCKET_COUNT) * 2, "Azure should have ${int(BUCKET_COUNT) * 2} resources (account + container)"
