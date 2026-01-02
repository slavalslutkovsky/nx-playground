"""
Unit Tests for Multi-Cloud Network Infrastructure

Tests the network composition functions for AWS VPC, GCP VPC, and Azure VNet
including security groups, firewalls, NAT gateways, and flow logs.
"""

import ...functions.network.main as network

# ============================================================================
# Test Data
# ============================================================================

# Mock OXR for AWS
_aws_oxr = {
    metadata = {
        name = "test-network"
        labels = {
            "${network.LABEL_AWS}": "true"
        }
    }
    spec = {
        parameters = {
            region = "us-east-1"
            cidrBlock = "10.0.0.0/16"
        }
    }
}

# Mock OXR for GCP
_gcp_oxr = {
    metadata = {
        name = "test-network"
        labels = {
            "${network.LABEL_GCP}": "true"
        }
    }
    spec = {
        parameters = {
            region = "us-central1"
            cidrBlock = "10.0.0.0/16"
        }
    }
}

# Mock OXR for Azure
_azure_oxr = {
    metadata = {
        name = "test-network"
        labels = {
            "${network.LABEL_AZURE}": "true"
        }
    }
    spec = {
        parameters = {
            region = "eastus"
            cidrBlock = "10.0.0.0/16"
            azure = {
                resourceGroup = "test-rg"
            }
        }
    }
}

# Mock OXR for multi-cloud
_multi_oxr = {
    metadata = {
        name = "test-network"
        labels = {
            "${network.LABEL_AWS}": "true"
            "${network.LABEL_GCP}": "true"
            "${network.LABEL_AZURE}": "true"
        }
    }
    spec = {
        parameters = {
            region = "us-east-1"
            cidrBlock = "10.0.0.0/16"
            azure = {
                resourceGroup = "test-rg"
            }
        }
    }
}

# Base network parameters for testing
_base_params = network.NetworkParams {
    name = "test-network"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    tags = {"Environment": "test"}
}

# AWS parameters
_aws_params = network.NetworkParams {
    name = "test-network"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    tags = {"Environment": "test"}
    aws = network.AWSConfig {}
}

# GCP parameters
_gcp_params = network.NetworkParams {
    name = "test-network"
    region = "us-central1"
    cidrBlock = "10.0.0.0/16"
    tags = {"Environment": "test"}
    gcp = network.GCPConfig {
        project = "test-project"
    }
}

# Azure parameters
_azure_params = network.NetworkParams {
    name = "test-network"
    region = "eastus"
    cidrBlock = "10.0.0.0/16"
    tags = {"Environment": "test"}
    azure = network.AzureConfig {
        resourceGroup = "test-rg"
    }
}

# ============================================================================
# AWS VPC Tests
# ============================================================================

_aws_resources = network.createAWSResources(_aws_params)

# Test: AWS VPC is created
_aws_vpcs = [r for r in _aws_resources if r?.kind == "VPC"]
assert len(_aws_vpcs) == 1, "Expected 1 AWS VPC"
assert _aws_vpcs[0].metadata.name == "test-network-aws", "VPC name mismatch"
assert _aws_vpcs[0].spec.forProvider.cidrBlock == "10.0.0.0/16", "VPC CIDR mismatch"

# Test: AWS Internet Gateway is created
_aws_igws = [r for r in _aws_resources if r?.kind == "InternetGateway"]
assert len(_aws_igws) == 1, "Expected 1 Internet Gateway"

# Test: AWS Public Subnets are created (default 3)
_aws_public_subnets = [r for r in _aws_resources if r?.kind == "Subnet" and "public" in r.metadata.name]
assert len(_aws_public_subnets) == 3, "Expected 3 public subnets"

# Test: AWS Private Subnets are created (default 3)
_aws_private_subnets = [r for r in _aws_resources if r?.kind == "Subnet" and "private" in r.metadata.name]
assert len(_aws_private_subnets) == 3, "Expected 3 private subnets"

# Test: AWS NAT Gateway is created (default enabled, single)
_aws_nat_gws = [r for r in _aws_resources if r?.kind == "NATGateway"]
assert len(_aws_nat_gws) == 1, "Expected 1 NAT Gateway (non-HA mode)"

# Test: AWS Security Group is created
_aws_sgs = [r for r in _aws_resources if r?.kind == "SecurityGroup"]
assert len(_aws_sgs) == 1, "Expected 1 Security Group"

# Test: AWS Security Group Rules are created
_aws_sg_rules = [r for r in _aws_resources if r?.kind == "SecurityGroupRule"]
assert len(_aws_sg_rules) >= 2, "Expected at least 2 Security Group Rules (internal + egress)"

# Test: AWS Route Tables are created
_aws_rts = [r for r in _aws_resources if r?.kind == "RouteTable"]
assert len(_aws_rts) >= 2, "Expected at least 2 Route Tables (public + private)"

# ============================================================================
# GCP VPC Tests
# ============================================================================

_gcp_resources = network.createGCPResources(_gcp_params)

# Test: GCP Network is created
_gcp_networks = [r for r in _gcp_resources if r?.kind == "Network"]
assert len(_gcp_networks) == 1, "Expected 1 GCP Network"
assert _gcp_networks[0].metadata.name == "test-network-gcp", "Network name mismatch"
assert _gcp_networks[0].spec.forProvider.autoCreateSubnetworks == False, "Should not auto-create subnetworks"

# Test: GCP Subnetworks are created
_gcp_subnets = [r for r in _gcp_resources if r?.kind == "Subnetwork"]
assert len(_gcp_subnets) == 6, "Expected 6 GCP Subnetworks (3 public + 3 private)"

# Test: GCP Cloud Router is created
_gcp_routers = [r for r in _gcp_resources if r?.kind == "Router"]
assert len(_gcp_routers) == 1, "Expected 1 Cloud Router"

# Test: GCP Cloud NAT is created
_gcp_nats = [r for r in _gcp_resources if r?.kind == "RouterNAT"]
assert len(_gcp_nats) == 1, "Expected 1 Cloud NAT"

# Test: GCP Firewalls are created
_gcp_fws = [r for r in _gcp_resources if r?.kind == "Firewall"]
assert len(_gcp_fws) >= 4, "Expected at least 4 Firewalls (internal, egress, deny-ssh, allow-iap)"

# Test: GCP Firewall denies SSH from internet
_gcp_deny_ssh = [r for r in _gcp_fws if "deny-ssh" in r.metadata.name]
assert len(_gcp_deny_ssh) == 1, "Expected 1 deny-ssh firewall rule"

# Test: GCP Firewall allows IAP
_gcp_allow_iap = [r for r in _gcp_fws if "allow-iap" in r.metadata.name]
assert len(_gcp_allow_iap) == 1, "Expected 1 allow-iap firewall rule"

# ============================================================================
# Azure VNet Tests
# ============================================================================

_azure_resources = network.createAzureResources(_azure_params)

# Test: Azure VNet is created
_azure_vnets = [r for r in _azure_resources if r?.kind == "VirtualNetwork"]
assert len(_azure_vnets) == 1, "Expected 1 Azure VNet"
assert _azure_vnets[0].metadata.name == "test-network-azure", "VNet name mismatch"

# Test: Azure Subnets are created
_azure_subnets = [r for r in _azure_resources if r?.kind == "Subnet"]
assert len(_azure_subnets) == 6, "Expected 6 Azure Subnets (3 public + 3 private)"

# Test: Azure NAT Gateway resources are created
_azure_nat_gws = [r for r in _azure_resources if r?.kind == "NATGateway"]
assert len(_azure_nat_gws) == 1, "Expected 1 NAT Gateway"

_azure_nat_pips = [r for r in _azure_resources if r?.kind == "PublicIP" and "nat" in r.metadata.name]
assert len(_azure_nat_pips) == 1, "Expected 1 NAT Public IP"

# Test: Azure NSG is created
_azure_nsgs = [r for r in _azure_resources if r?.kind == "SecurityGroup"]
assert len(_azure_nsgs) == 1, "Expected 1 Network Security Group"

# Test: Azure Security Rules are created
_azure_rules = [r for r in _azure_resources if r?.kind == "SecurityRule"]
assert len(_azure_rules) >= 4, "Expected at least 4 Security Rules"

# Test: Azure denies SSH/RDP from internet
_azure_deny_ssh = [r for r in _azure_rules if "deny-ssh" in r.metadata.name]
assert len(_azure_deny_ssh) == 1, "Expected 1 deny-ssh security rule"

_azure_deny_rdp = [r for r in _azure_rules if "deny-rdp" in r.metadata.name]
assert len(_azure_deny_rdp) == 1, "Expected 1 deny-rdp security rule"

# ============================================================================
# Multi-Cloud Tests
# ============================================================================

_multi_resources = network.createNetwork(_multi_oxr, {}, network.NetworkParams {
    name = "test-network"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    aws = network.AWSConfig {}
    gcp = network.GCPConfig {project = "test"}
    azure = network.AzureConfig {resourceGroup = "test-rg"}
})

# Test: Resources are created for all providers
_multi_aws = network.filterByProvider(_multi_resources, "aws")
_multi_gcp = network.filterByProvider(_multi_resources, "gcp")
_multi_azure = network.filterByProvider(_multi_resources, "azure")

assert len(_multi_aws) > 0, "Expected AWS resources in multi-cloud"
assert len(_multi_gcp) > 0, "Expected GCP resources in multi-cloud"
assert len(_multi_azure) > 0, "Expected Azure resources in multi-cloud"

# ============================================================================
# Utility Function Tests
# ============================================================================

# Test: getActiveProviders
_aws_only_labels = {"${network.LABEL_AWS}": "true"}
_all_labels = {
    "${network.LABEL_AWS}": "true"
    "${network.LABEL_GCP}": "true"
    "${network.LABEL_AZURE}": "true"
}

_active_aws = network.getActiveProviders(_aws_only_labels)
assert _active_aws == ["aws"], "Expected only AWS provider"

_active_all = network.getActiveProviders(_all_labels)
assert len(_active_all) == 3, "Expected 3 providers"
assert "aws" in _active_all, "Expected aws in providers"
assert "gcp" in _active_all, "Expected gcp in providers"
assert "azure" in _active_all, "Expected azure in providers"

# Test: filterByProvider
_filtered_aws = network.filterByProvider(_multi_resources, "aws")
assert all(r.metadata.labels.get(network.LABEL_AWS, "false") == "true" for r in _filtered_aws), "All filtered resources should have AWS label"

# Test: filterBySubnetType
_public_filtered = network.filterBySubnetType(_aws_resources, "public")
_private_filtered = network.filterBySubnetType(_aws_resources, "private")

assert len(_public_filtered) > 0, "Expected public subnets"
assert len(_private_filtered) > 0, "Expected private subnets"

# Test: calculateSubnetCidr
_cidr_1 = network.calculateSubnetCidr(0, 0, 24)
_cidr_2 = network.calculateSubnetCidr(1, 2, 24)
assert _cidr_1 == "10.0.0.0/24", "First subnet CIDR mismatch"
assert _cidr_2 == "10.0.18.0/24", "Second subnet CIDR mismatch"

# ============================================================================
# Schema Validation Tests
# ============================================================================

# Test: NetworkParams validation
_valid_params = network.NetworkParams {
    name = "valid-network"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
}
assert _valid_params.name == "valid-network", "Valid params should be created"

# Test: SubnetTypeConfig validation
_valid_subnet = network.SubnetTypeConfig {
    count = 3
    cidrMask = 24
}
assert _valid_subnet.count == 3, "Subnet count should be 3"

# Test: FirewallRule validation
_valid_fw_rule = network.FirewallRule {
    name = "allow-http"
    priority = 1000
    direction = "ingress"
    action = "allow"
    protocol = "tcp"
    ports = ["80", "443"]
    sourceCidrs = ["0.0.0.0/0"]
}
assert _valid_fw_rule.direction == "ingress", "Firewall direction should be ingress"

# ============================================================================
# Security Configuration Tests
# ============================================================================

# Test: Database subnets disabled by default
_no_db_params = network.NetworkParams {
    name = "test-no-db"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    aws = network.AWSConfig {}
}
_no_db_resources = network.createAWSResources(_no_db_params)
_no_db_subnets = [r for r in _no_db_resources if r?.kind == "Subnet" and "database" in r.metadata.name]
assert len(_no_db_subnets) == 0, "Database subnets should be disabled by default"

# Test: Database subnets enabled
_db_params = network.NetworkParams {
    name = "test-with-db"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    subnets = network.SubnetConfig {
        database = network.DatabaseSubnetConfig {
            enabled = True
            count = 2
        }
    }
    aws = network.AWSConfig {}
}
_db_resources = network.createAWSResources(_db_params)
_db_subnets = [r for r in _db_resources if r?.kind == "Subnet" and "database" in r.metadata.name]
assert len(_db_subnets) == 2, "Expected 2 database subnets when enabled"

# ============================================================================
# High Availability Tests
# ============================================================================

# Test: HA NAT Gateways
_ha_params = network.NetworkParams {
    name = "test-ha"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    connectivity = network.ConnectivityConfig {
        natGateway = network.NatGatewayConfig {
            enabled = True
            highAvailability = True
        }
    }
    aws = network.AWSConfig {}
}
_ha_resources = network.createAWSResources(_ha_params)
_ha_nat_gws = [r for r in _ha_resources if r?.kind == "NATGateway"]
assert len(_ha_nat_gws) == 3, "Expected 3 NAT Gateways in HA mode (one per AZ)"

# ============================================================================
# Observability Tests
# ============================================================================

# Test: Flow logs disabled by default
_no_flow_resources = network.createAWSResources(_aws_params)
_no_flow_logs = [r for r in _no_flow_resources if r?.kind == "FlowLog"]
assert len(_no_flow_logs) == 0, "Flow logs should be disabled by default"

# Test: Flow logs enabled
_flow_params = network.NetworkParams {
    name = "test-flow"
    region = "us-east-1"
    cidrBlock = "10.0.0.0/16"
    observability = network.ObservabilityConfig {
        flowLogs = network.FlowLogConfig {
            enabled = True
            trafficType = "ALL"
        }
    }
    aws = network.AWSConfig {}
}
_flow_resources = network.createAWSResources(_flow_params)
_flow_logs = [r for r in _flow_resources if r?.kind == "FlowLog"]
assert len(_flow_logs) == 1, "Expected 1 Flow Log when enabled"

# ============================================================================
# All Tests Passed
# ============================================================================

_test_summary = {
    aws_tests = "PASSED"
    gcp_tests = "PASSED"
    azure_tests = "PASSED"
    multi_cloud_tests = "PASSED"
    utility_tests = "PASSED"
    schema_tests = "PASSED"
    security_tests = "PASSED"
    ha_tests = "PASSED"
    observability_tests = "PASSED"
}

print("All network unit tests passed!")
print(_test_summary)
