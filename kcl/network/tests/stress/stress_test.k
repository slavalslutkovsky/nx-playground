"""
Stress Tests for Multi-Cloud Network Infrastructure

Tests the network composition functions at scale, generating multiple
networks across all providers to validate performance and resource limits.
"""

import ...functions.network.main as network

# ============================================================================
# Configuration
# ============================================================================

# Number of networks to generate (can be overridden with -D "NETWORK_COUNT=X")
NETWORK_COUNT = option("NETWORK_COUNT") or 50

# ============================================================================
# Generate Test Networks
# ============================================================================

# Generate networks for each provider
_aws_networks = [
    network.createAWSResources(network.NetworkParams {
        name = "stress-test-${i}"
        region = "us-east-1"
        cidrBlock = "10.${i}.0.0/16"
        tags = {
            "StressTest": "true"
            "Index": "${i}"
        }
        subnets = network.SubnetConfig {
            public = network.SubnetTypeConfig {count = 2, cidrMask = 24}
            private = network.SubnetTypeConfig {count = 2, cidrMask = 24}
        }
        connectivity = network.ConnectivityConfig {
            natGateway = network.NatGatewayConfig {enabled = True}
        }
        security = network.SecurityConfig {
            defaultSecurityGroup = network.DefaultSecurityGroupConfig {enabled = True}
        }
        aws = network.AWSConfig {}
    })
    for i in range(int(NETWORK_COUNT))
]

_gcp_networks = [
    network.createGCPResources(network.NetworkParams {
        name = "stress-test-${i}"
        region = "us-central1"
        cidrBlock = "10.${i}.0.0/16"
        tags = {
            "StressTest": "true"
            "Index": "${i}"
        }
        subnets = network.SubnetConfig {
            public = network.SubnetTypeConfig {count = 2, cidrMask = 24}
            private = network.SubnetTypeConfig {count = 2, cidrMask = 24}
        }
        gcp = network.GCPConfig {
            project = "stress-test-project"
        }
    })
    for i in range(int(NETWORK_COUNT))
]

_azure_networks = [
    network.createAzureResources(network.NetworkParams {
        name = "stress-test-${i}"
        region = "eastus"
        cidrBlock = "10.${i}.0.0/16"
        tags = {
            "StressTest": "true"
            "Index": "${i}"
        }
        subnets = network.SubnetConfig {
            public = network.SubnetTypeConfig {count = 2, cidrMask = 24}
            private = network.SubnetTypeConfig {count = 2, cidrMask = 24}
        }
        azure = network.AzureConfig {
            resourceGroup = "stress-test-rg"
        }
    })
    for i in range(int(NETWORK_COUNT))
]

# Flatten all resources
_all_aws_resources = [r for resources in _aws_networks for r in resources]
_all_gcp_resources = [r for resources in _gcp_networks for r in resources]
_all_azure_resources = [r for resources in _azure_networks for r in resources]

# ============================================================================
# Assertions
# ============================================================================

# Count resources per provider
_aws_count = len(_all_aws_resources)
_gcp_count = len(_all_gcp_resources)
_azure_count = len(_all_azure_resources)
_total_count = _aws_count + _gcp_count + _azure_count

# Verify minimum expected resources per network
# AWS: VPC + IGW + 2 public + 2 private + 1 NAT + 1 EIP + 2 RTs + routes + SG + SG rules + associations
# GCP: Network + 4 subnets + router + NAT + 4+ firewalls
# Azure: VNet + 4 subnets + NAT + PIP + NSG + rules + RT

_min_aws_per_network = 15  # Conservative minimum
_min_gcp_per_network = 10
_min_azure_per_network = 12

assert _aws_count >= int(NETWORK_COUNT) * _min_aws_per_network, \
    "Expected at least ${int(NETWORK_COUNT) * _min_aws_per_network} AWS resources, got ${_aws_count}"

assert _gcp_count >= int(NETWORK_COUNT) * _min_gcp_per_network, \
    "Expected at least ${int(NETWORK_COUNT) * _min_gcp_per_network} GCP resources, got ${_gcp_count}"

assert _azure_count >= int(NETWORK_COUNT) * _min_azure_per_network, \
    "Expected at least ${int(NETWORK_COUNT) * _min_azure_per_network} Azure resources, got ${_azure_count}"

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
    network_count = int(NETWORK_COUNT)
    aws_resources = _aws_count
    gcp_resources = _gcp_count
    azure_resources = _azure_count
    total_resources = _total_count
    avg_per_aws_network = _aws_count // int(NETWORK_COUNT)
    avg_per_gcp_network = _gcp_count // int(NETWORK_COUNT)
    avg_per_azure_network = _azure_count // int(NETWORK_COUNT)
}

print("Stress Test Results:")
print("====================")
print("Networks per provider: ${NETWORK_COUNT}")
print("AWS resources: ${_aws_count}")
print("GCP resources: ${_gcp_count}")
print("Azure resources: ${_azure_count}")
print("Total resources: ${_total_count}")
print("")
print("Stress test passed!")
