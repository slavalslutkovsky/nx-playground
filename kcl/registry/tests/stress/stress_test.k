"""
Stress Tests for Multi-Cloud Container Registry

Tests the registry composition functions at scale, generating multiple
registries across all providers to validate performance and resource limits.
"""

import ...functions.registry.main as registry

# ============================================================================
# Configuration
# ============================================================================

# Number of registries to generate (can be overridden with -D "REGISTRY_COUNT=X")
REGISTRY_COUNT = option("REGISTRY_COUNT") or 50

# ============================================================================
# Generate Test Registries
# ============================================================================

# Generate registries for each provider
_aws_registries = [
    registry.createAWSResources(registry.RegistryParams {
        name = "stress-test-${i}"
        region = "us-east-1"
        repositoryName = "stress-repo-${i}"
        tags = {
            "StressTest": "true"
            "Index": "${i}"
        }
        security = registry.SecurityConfig {
            scanOnPush = True
        }
        lifecycle = registry.LifecycleConfig {
            enabled = True
            maxImageAge = 7
            maxImageCount = 50
        }
        aws = registry.AWSConfig {}
    })
    for i in range(int(REGISTRY_COUNT))
]

_gcp_registries = [
    registry.createGCPResources(registry.RegistryParams {
        name = "stress-test-${i}"
        region = "us-central1"
        repositoryName = "stress-repo-${i}"
        tags = {
            "StressTest": "true"
            "Index": "${i}"
        }
        gcp = registry.GCPConfig {
            project = "stress-test-project"
            format = "DOCKER"
        }
    })
    for i in range(int(REGISTRY_COUNT))
]

_azure_registries = [
    registry.createAzureResources(registry.RegistryParams {
        name = "stress-test-${i}"
        region = "eastus"
        repositoryName = "stressrepo${i}"
        tags = {
            "StressTest": "true"
            "Index": "${i}"
        }
        azure = registry.AzureConfig {
            resourceGroup = "stress-test-rg"
            sku = "Standard"
        }
    })
    for i in range(int(REGISTRY_COUNT))
]

# Flatten all resources
_all_aws_resources = [r for resources in _aws_registries for r in resources]
_all_gcp_resources = [r for resources in _gcp_registries for r in resources]
_all_azure_resources = [r for resources in _azure_registries for r in resources]

# ============================================================================
# Assertions
# ============================================================================

# Count resources per provider
_aws_count = len(_all_aws_resources)
_gcp_count = len(_all_gcp_resources)
_azure_count = len(_all_azure_resources)
_total_count = _aws_count + _gcp_count + _azure_count

# Verify minimum expected resources per registry
# AWS: Repository + LifecyclePolicy = 2
# GCP: RegistryRepository = 1
# Azure: Registry = 1

_min_aws_per_registry = 2
_min_gcp_per_registry = 1
_min_azure_per_registry = 1

assert _aws_count >= int(REGISTRY_COUNT) * _min_aws_per_registry, \
    "Expected at least ${int(REGISTRY_COUNT) * _min_aws_per_registry} AWS resources, got ${_aws_count}"

assert _gcp_count >= int(REGISTRY_COUNT) * _min_gcp_per_registry, \
    "Expected at least ${int(REGISTRY_COUNT) * _min_gcp_per_registry} GCP resources, got ${_gcp_count}"

assert _azure_count >= int(REGISTRY_COUNT) * _min_azure_per_registry, \
    "Expected at least ${int(REGISTRY_COUNT) * _min_azure_per_registry} Azure resources, got ${_azure_count}"

# Verify all resources have valid metadata
_all_have_names = all(r?.metadata?.name for r in _all_aws_resources + _all_gcp_resources + _all_azure_resources)
assert _all_have_names, "All resources must have metadata.name"

# Verify all resources have provider labels
_all_have_labels = all(r?.metadata?.labels for r in _all_aws_resources + _all_gcp_resources + _all_azure_resources)
assert _all_have_labels, "All resources must have metadata.labels"

# ============================================================================
# Results Summary
# ============================================================================

_summary = {
    registry_count = int(REGISTRY_COUNT)
    aws_resources = _aws_count
    gcp_resources = _gcp_count
    azure_resources = _azure_count
    total_resources = _total_count
    avg_per_aws_registry = _aws_count // int(REGISTRY_COUNT)
    avg_per_gcp_registry = _gcp_count // int(REGISTRY_COUNT)
    avg_per_azure_registry = _azure_count // int(REGISTRY_COUNT)
}

print("Stress Test Results:")
print("====================")
print("Registries per provider: ${REGISTRY_COUNT}")
print("AWS resources: ${_aws_count}")
print("GCP resources: ${_gcp_count}")
print("Azure resources: ${_azure_count}")
print("Total resources: ${_total_count}")
print("")
print("Stress test passed!")
