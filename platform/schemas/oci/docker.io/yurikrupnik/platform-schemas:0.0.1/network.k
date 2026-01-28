"""
Network schema - Provider-agnostic VPC/Network specification.
Source of truth for XNetwork XRD and compose-network function.

Creates:
- AWS: VPC + Subnets + InternetGateway + RouteTables
- GCP: Network + Subnetworks

Default behavior: Creates private and public subnets in Europe if not specified.
"""
import .common as c

type SubnetType = "public" | "private"

schema SubnetSpec:
    """Provider-agnostic subnet specification."""
    name: str
    cidrBlock: str
    type: SubnetType = "private"
    availabilityZone?: str  # Optional, for AWS multi-AZ

    check:
        len(name) > 0, "subnet name is required"
        len(cidrBlock) > 0, "subnet cidrBlock is required"

schema NetworkSpec(c.BaseSpec):
    """Provider-agnostic network/VPC specification."""
    provider: c.CloudProvider
    name: str
    region: str = "europe-west1"  # Default to Europe
    cidrBlock: str = "10.0.0.0/16"
    subnets?: [SubnetSpec]
    enableDnsSupport: bool = True
    enableDnsHostnames: bool = True

    check:
        len(name) > 0, "name is required"

schema NetworkStatus(c.BaseStatus):
    """Status fields for network resources."""
    networkId?: str
    vpcId?: str
    cidrBlock?: str
    subnetIds?: [str]

# Default subnets for Europe (used when subnets not specified)
NETWORK_DEFAULT_SUBNETS_GCP = [
    {name = "public", cidrBlock = "10.0.1.0/24", type = "public"}
    {name = "private", cidrBlock = "10.0.2.0/24", type = "private"}
]

NETWORK_DEFAULT_SUBNETS_AWS = [
    {name = "public-a", cidrBlock = "10.0.1.0/24", type = "public", availabilityZone = "a"}
    {name = "public-b", cidrBlock = "10.0.2.0/24", type = "public", availabilityZone = "b"}
    {name = "private-a", cidrBlock = "10.0.10.0/24", type = "private", availabilityZone = "a"}
    {name = "private-b", cidrBlock = "10.0.11.0/24", type = "private", availabilityZone = "b"}
]

# Region mapping: generic region to cloud-specific
NETWORK_REGION_MAP_GCP = {
    "europe-west1": "europe-west1"
    "europe-west2": "europe-west2"
    "us-east1": "us-east1"
    "us-west1": "us-west1"
}

NETWORK_REGION_MAP_AWS = {
    "europe-west1": "eu-west-1"
    "europe-west2": "eu-west-2"
    "us-east1": "us-east-1"
    "us-west1": "us-west-1"
}

# OpenAPI v3 schema for XRD generation
NETWORK_SUBNET_SCHEMA = {
    type = "object"
    description = "Subnet configuration"
    properties = {
        name = {type = "string", description = "Subnet name"}
        cidrBlock = {type = "string", description = "CIDR block for the subnet (e.g., 10.0.1.0/24)"}
        type = {type = "string", enum = ["public", "private"], default = "private", description = "Subnet type"}
        availabilityZone = {type = "string", description = "Availability zone suffix (e.g., 'a', 'b') - AWS only"}
    }
    required = ["name", "cidrBlock"]
}

NETWORK_SPEC_PROPERTIES = {
    provider = c.PROVIDER_CLOUD_SCHEMA
    region = {
        type = "string"
        description = "Cloud region (e.g., europe-west1, us-east1). Mapped to provider-specific region."
        default = "europe-west1"
    }
    name = {
        type = "string"
        description = "Name of the network/VPC"
    }
    cidrBlock = {
        type = "string"
        description = "CIDR block for the VPC (e.g., 10.0.0.0/16)"
        default = "10.0.0.0/16"
    }
    subnets = {
        type = "array"
        description = "Array of subnet configurations. If not specified, default public/private subnets are created."
        items = NETWORK_SUBNET_SCHEMA
    }
    enableDnsSupport = c._boolean_schema("Enable DNS support in the VPC", True)
    enableDnsHostnames = c._boolean_schema("Enable DNS hostnames in the VPC", True)
    environment = c.ENVIRONMENT_SCHEMA
    deletionPolicy = c.DELETION_POLICY_SCHEMA
}

NETWORK_SPEC_REQUIRED = ["provider", "name"]

NETWORK_STATUS_PROPERTIES = {
    provider = {type = "string"}
    region = {type = "string"}
    networkId = {type = "string", description = "GCP Network ID or AWS VPC ID"}
    vpcId = {type = "string", description = "AWS VPC ID (alias for networkId)"}
    subnetIds = {
        type = "array"
        items = {type = "string"}
        description = "List of created subnet IDs"
    }
}

# XRD metadata
NETWORK_XRD_NAME = "xnetworks.network.platform.io"
NETWORK_XRD_GROUP = "network.platform.io"
NETWORK_XRD_KIND = "XNetwork"
NETWORK_XRD_PLURAL = "xnetworks"
NETWORK_XRD_CLAIM_KIND = "Network"
NETWORK_XRD_CLAIM_PLURAL = "networks"
NETWORK_XRD_COMPOSITION = "xnetwork"
