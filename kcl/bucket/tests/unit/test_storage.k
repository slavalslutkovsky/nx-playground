"""
Unit tests for the unified storage function.
Tests parameter validation and resource generation logic.
"""

import functions.storage.main as storage

# Test: AWS resource creation
_test_aws_params = storage.StorageParams {
    name = "test-bucket"
    location = "us-east-1"
    acl = "private"
    versioning = True
    tags = {"env": "test"}
    aws = storage.AWSConfig {
        region = "us-east-1"
    }
}

_aws_resources = storage.createAWSResources(_test_aws_params)

# Assertions for AWS
assert len(_aws_resources) >= 2, "Should create at least bucket and ACL"
assert _aws_resources[0].kind == "Bucket", "First resource should be Bucket"
assert _aws_resources[0].metadata.labels["cloud.provider/aws"] == "true", "Should have AWS label"

# Test: GCP resource creation
_test_gcp_params = storage.StorageParams {
    name = "test-bucket"
    location = "US"
    acl = "private"
    versioning = False
    tags = {"env": "test"}
    gcp = storage.GCPConfig {
        storageClass = "STANDARD"
    }
}

_gcp_resources = storage.createGCPResources(_test_gcp_params)

# Assertions for GCP
assert len(_gcp_resources) == 1, "Should create one bucket"
assert _gcp_resources[0].kind == "Bucket", "Should be a Bucket"
assert _gcp_resources[0].metadata.labels["cloud.provider/gcp"] == "true", "Should have GCP label"

# Test: Azure resource creation
_test_azure_params = storage.StorageParams {
    name = "test-bucket"
    location = "eastus"
    acl = "private"
    versioning = True
    tags = {"env": "test"}
    azure = storage.AzureConfig {
        resourceGroup = "test-rg"
        accountTier = "Standard"
        accountReplicationType = "LRS"
    }
}

_azure_resources = storage.createAzureResources(_test_azure_params)

# Assertions for Azure
assert len(_azure_resources) == 2, "Should create account and container"
assert _azure_resources[0].kind == "Account", "First should be Account"
assert _azure_resources[1].kind == "Container", "Second should be Container"
assert _azure_resources[0].metadata.labels["cloud.provider/azure"] == "true", "Should have Azure label"

# Test: Provider filtering utility
_test_labels_aws = {"cloud.provider/aws": "true", "cloud.provider/gcp": "false"}
_active_providers_aws = storage.getActiveProviders(_test_labels_aws)
assert _active_providers_aws == ["aws"], "Should return only AWS"

_test_labels_multi = {"cloud.provider/aws": "true", "cloud.provider/gcp": "true", "cloud.provider/azure": "true"}
_active_providers_multi = storage.getActiveProviders(_test_labels_multi)
assert len(_active_providers_multi) == 3, "Should return all three providers"

# Test: Main composition function with mock OXR
_mock_oxr_aws = {
    metadata = {
        name = "test-aws-bucket"
        labels = {"cloud.provider/aws": "true"}
    }
    spec = {
        parameters = {
            location = "us-west-2"
            acl = "private"
            version = False
        }
    }
}

_composition_params = storage.StorageParams {
    name = "test-aws-bucket"
    location = "us-west-2"
    aws = storage.AWSConfig {
        region = "us-west-2"
    }
}

_composition_result = storage.createStorageBucket(_mock_oxr_aws, {}, _composition_params)

assert len(_composition_result) >= 1, "Should create AWS resources"

# All tests passed
test_results = {
    aws_tests = "PASSED"
    gcp_tests = "PASSED"
    azure_tests = "PASSED"
    utility_tests = "PASSED"
    composition_tests = "PASSED"
}
