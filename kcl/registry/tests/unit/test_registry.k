"""
Unit Tests for Multi-Cloud Container Registry

Tests the registry composition functions for AWS ECR, GCP Artifact Registry,
and Azure Container Registry including security scanning, lifecycle policies,
and access control.
"""

import ...functions.registry.main as registry

# ============================================================================
# Test Data
# ============================================================================

# Base registry parameters for testing
_base_params = registry.RegistryParams {
    name = "test-registry"
    region = "us-east-1"
    repositoryName = "my-app"
    tags = {"Environment": "test"}
}

# AWS parameters
_aws_params = registry.RegistryParams {
    name = "test-registry"
    region = "us-east-1"
    repositoryName = "my-app"
    tags = {"Environment": "test"}
    security = registry.SecurityConfig {
        scanOnPush = True
        encryptionType = "AES256"
    }
    lifecycle = registry.LifecycleConfig {
        enabled = True
        maxImageAge = 14
        maxImageCount = 100
    }
    aws = registry.AWSConfig {
        forceDelete = False
        scanType = "BASIC"
    }
}

# GCP parameters
_gcp_params = registry.RegistryParams {
    name = "test-registry"
    region = "us-central1"
    repositoryName = "my-app"
    tags = {"Environment": "test"}
    gcp = registry.GCPConfig {
        project = "test-project"
        format = "DOCKER"
        mode = "STANDARD_REPOSITORY"
    }
}

# Azure parameters
_azure_params = registry.RegistryParams {
    name = "test-registry"
    region = "eastus"
    repositoryName = "my-app"
    tags = {"Environment": "test"}
    azure = registry.AzureConfig {
        resourceGroup = "test-rg"
        sku = "Standard"
    }
}

# ============================================================================
# AWS ECR Tests
# ============================================================================

_aws_resources = registry.createAWSResources(_aws_params)

# Test: AWS ECR Repository is created
_aws_repos = [r for r in _aws_resources if r?.kind == "Repository"]
assert len(_aws_repos) == 1, "Expected 1 AWS ECR Repository"
assert _aws_repos[0].metadata.name == "my-app-aws", "Repository name mismatch"
assert _aws_repos[0].spec.forProvider.imageTagMutability == "MUTABLE", "Image tag mutability mismatch"

# Test: AWS Lifecycle Policy is created
_aws_lifecycle = [r for r in _aws_resources if r?.kind == "LifecyclePolicy"]
assert len(_aws_lifecycle) == 1, "Expected 1 Lifecycle Policy"

# Test: Provider label is set
assert _aws_repos[0].metadata.labels.get(registry.LABEL_AWS) == "true", "AWS label not set"

# ============================================================================
# GCP Artifact Registry Tests
# ============================================================================

_gcp_resources = registry.createGCPResources(_gcp_params)

# Test: GCP Repository is created
_gcp_repos = [r for r in _gcp_resources if r?.kind == "RegistryRepository"]
assert len(_gcp_repos) == 1, "Expected 1 GCP Artifact Registry Repository"
assert _gcp_repos[0].metadata.name == "my-app-gcp", "Repository name mismatch"
assert _gcp_repos[0].spec.forProvider.format == "DOCKER", "Format mismatch"

# Test: Provider label is set
assert _gcp_repos[0].metadata.labels.get(registry.LABEL_GCP) == "true", "GCP label not set"

# ============================================================================
# Azure Container Registry Tests
# ============================================================================

_azure_resources = registry.createAzureResources(_azure_params)

# Test: Azure Registry is created
_azure_regs = [r for r in _azure_resources if r?.kind == "Registry"]
assert len(_azure_regs) == 1, "Expected 1 Azure Container Registry"
# ACR names are alphanumeric only
assert _azure_regs[0].metadata.name == "myapp", "Registry name should be alphanumeric"
assert _azure_regs[0].spec.forProvider.sku == "Standard", "SKU mismatch"

# Test: Provider label is set
assert _azure_regs[0].metadata.labels.get(registry.LABEL_AZURE) == "true", "Azure label not set"

# ============================================================================
# Multi-Cloud Tests
# ============================================================================

# Mock OXR for multi-cloud
_multi_oxr = {
    metadata = {
        name = "test-registry"
        labels = {
            "${registry.LABEL_AWS}": "true"
            "${registry.LABEL_GCP}": "true"
            "${registry.LABEL_AZURE}": "true"
        }
    }
    spec = {
        parameters = {
            region = "us-east-1"
            repositoryName = "multi-app"
        }
    }
}

_multi_params = registry.RegistryParams {
    name = "test-registry"
    region = "us-east-1"
    repositoryName = "multi-app"
    aws = registry.AWSConfig {}
    gcp = registry.GCPConfig {project = "test"}
    azure = registry.AzureConfig {resourceGroup = "test-rg"}
}

_multi_resources = registry.createContainerRegistry(_multi_oxr, {}, _multi_params)

# Test: Resources are created for all providers
_multi_aws = registry.filterByProvider(_multi_resources, "aws")
_multi_gcp = registry.filterByProvider(_multi_resources, "gcp")
_multi_azure = registry.filterByProvider(_multi_resources, "azure")

assert len(_multi_aws) > 0, "Expected AWS resources in multi-cloud"
assert len(_multi_gcp) > 0, "Expected GCP resources in multi-cloud"
assert len(_multi_azure) > 0, "Expected Azure resources in multi-cloud"

# ============================================================================
# Utility Function Tests
# ============================================================================

# Test: getActiveProviders
_aws_only_labels = {"${registry.LABEL_AWS}": "true"}
_all_labels = {
    "${registry.LABEL_AWS}": "true"
    "${registry.LABEL_GCP}": "true"
    "${registry.LABEL_AZURE}": "true"
}

_active_aws = registry.getActiveProviders(_aws_only_labels)
assert _active_aws == ["aws"], "Expected only AWS provider"

_active_all = registry.getActiveProviders(_all_labels)
assert len(_active_all) == 3, "Expected 3 providers"
assert "aws" in _active_all, "Expected aws in providers"
assert "gcp" in _active_all, "Expected gcp in providers"
assert "azure" in _active_all, "Expected azure in providers"

# Test: filterByProvider
_filtered_aws = registry.filterByProvider(_multi_resources, "aws")
assert all(r.metadata.labels.get(registry.LABEL_AWS, "false") == "true" for r in _filtered_aws), \
    "All filtered resources should have AWS label"

# Test: getRegistryUrl
_aws_url = registry.getRegistryUrl("aws", "my-repo", "us-east-1", "")
assert "dkr.ecr.us-east-1.amazonaws.com" in _aws_url, "AWS URL format incorrect"

_gcp_url = registry.getRegistryUrl("gcp", "my-repo", "us-central1", "my-project")
assert "us-central1-docker.pkg.dev/my-project" in _gcp_url, "GCP URL format incorrect"

_azure_url = registry.getRegistryUrl("azure", "myrepo", "eastus", "")
assert "azurecr.io" in _azure_url, "Azure URL format incorrect"

# ============================================================================
# Security Configuration Tests
# ============================================================================

# Test: Scan on push enabled by default
_scan_params = registry.RegistryParams {
    name = "test-scan"
    region = "us-east-1"
    repositoryName = "scan-test"
    security = registry.SecurityConfig {
        scanOnPush = True
    }
    aws = registry.AWSConfig {}
}
_scan_resources = registry.createAWSResources(_scan_params)
_scan_repos = [r for r in _scan_resources if r?.kind == "Repository"]
assert _scan_repos[0].spec.forProvider.imageScanningConfiguration.scanOnPush == True, \
    "Scan on push should be enabled"

# Test: KMS encryption
_kms_params = registry.RegistryParams {
    name = "test-kms"
    region = "us-east-1"
    repositoryName = "kms-test"
    security = registry.SecurityConfig {
        encryptionType = "KMS"
        kmsKeyId = "arn:aws:kms:us-east-1:123456789:key/abc123"
    }
    aws = registry.AWSConfig {}
}
_kms_resources = registry.createAWSResources(_kms_params)
_kms_repos = [r for r in _kms_resources if r?.kind == "Repository"]
assert _kms_repos[0].spec.forProvider.encryptionConfiguration.encryptionType == "KMS", \
    "Encryption type should be KMS"

# ============================================================================
# Lifecycle Policy Tests
# ============================================================================

# Test: Lifecycle policy disabled
_no_lifecycle_params = registry.RegistryParams {
    name = "test-no-lifecycle"
    region = "us-east-1"
    repositoryName = "no-lifecycle-test"
    lifecycle = registry.LifecycleConfig {
        enabled = False
    }
    aws = registry.AWSConfig {}
}
_no_lifecycle_resources = registry.createAWSResources(_no_lifecycle_params)
_no_lifecycle_policies = [r for r in _no_lifecycle_resources if r?.kind == "LifecyclePolicy"]
assert len(_no_lifecycle_policies) == 0, "Lifecycle policy should be disabled"

# ============================================================================
# Replication Tests
# ============================================================================

# Test: Replication enabled
_repl_params = registry.RegistryParams {
    name = "test-replication"
    region = "us-east-1"
    repositoryName = "repl-test"
    replication = registry.ReplicationConfig {
        enabled = True
        destinationRegions = ["us-west-2", "eu-west-1"]
    }
    aws = registry.AWSConfig {}
}
_repl_resources = registry.createAWSResources(_repl_params)
_repl_configs = [r for r in _repl_resources if r?.kind == "ReplicationConfiguration"]
assert len(_repl_configs) == 1, "Expected 1 replication configuration"

# ============================================================================
# Access Control Tests
# ============================================================================

# Test: IAM members for GCP
_iam_params = registry.RegistryParams {
    name = "test-iam"
    region = "us-central1"
    repositoryName = "iam-test"
    accessControl = registry.AccessControlConfig {
        pullPrincipals = ["serviceAccount:pull@project.iam.gserviceaccount.com"]
        pushPrincipals = ["serviceAccount:push@project.iam.gserviceaccount.com"]
    }
    gcp = registry.GCPConfig {project = "test-project"}
}
_iam_resources = registry.createGCPResources(_iam_params)
_iam_members = [r for r in _iam_resources if r?.kind == "RegistryRepositoryIamMember"]
assert len(_iam_members) == 2, "Expected 2 IAM members (1 pull, 1 push)"

# ============================================================================
# Azure Premium Features Tests
# ============================================================================

# Test: Azure Premium with geo-replication
_premium_params = registry.RegistryParams {
    name = "test-premium"
    region = "eastus"
    repositoryName = "premium-test"
    replication = registry.ReplicationConfig {
        enabled = True
        destinationRegions = ["westus", "northeurope"]
    }
    azure = registry.AzureConfig {
        resourceGroup = "test-rg"
        sku = "Premium"
        zoneRedundancy = True
    }
}
_premium_resources = registry.createAzureResources(_premium_params)
_premium_regs = [r for r in _premium_resources if r?.kind == "Registry"]
assert _premium_regs[0].spec.forProvider.sku == "Premium", "SKU should be Premium"

_geo_repls = [r for r in _premium_resources if r?.kind == "Replication"]
assert len(_geo_repls) == 2, "Expected 2 geo-replications"

# ============================================================================
# Schema Validation Tests
# ============================================================================

# Test: Valid registry params
_valid_params = registry.RegistryParams {
    name = "valid-registry"
    region = "us-east-1"
    repositoryName = "valid-repo"
}
assert _valid_params.imageTagMutability == "MUTABLE", "Default mutability should be MUTABLE"

# Test: Security config defaults
_security_defaults = registry.SecurityConfig {}
assert _security_defaults.scanOnPush == True, "Scan on push should default to true"
assert _security_defaults.encryptionType == "AES256", "Encryption should default to AES256"

# Test: Lifecycle config defaults
_lifecycle_defaults = registry.LifecycleConfig {}
assert _lifecycle_defaults.enabled == True, "Lifecycle should be enabled by default"
assert _lifecycle_defaults.maxImageAge == 14, "Max image age should default to 14"

# ============================================================================
# All Tests Passed
# ============================================================================

_test_summary = {
    aws_tests = "PASSED"
    gcp_tests = "PASSED"
    azure_tests = "PASSED"
    multi_cloud_tests = "PASSED"
    utility_tests = "PASSED"
    security_tests = "PASSED"
    lifecycle_tests = "PASSED"
    replication_tests = "PASSED"
    access_control_tests = "PASSED"
    schema_tests = "PASSED"
}

print("All registry unit tests passed!")
print(_test_summary)
