"""
Multi-provider network composition.
Creates AWS VPC or GCP Network with subnets based on provider field.

AWS: VPC + Subnets + InternetGateway + RouteTable + Routes
GCP: Network + Subnetworks
"""

import schemas.regions as regions
import schemas.helpers as h
import schemas.mappings as m

# Crossplane function inputs
oxr = option("params").oxr
_ocds = option("params").ocds
dxr = option("params").dxr
dcds = option("params").dcds

# Extract spec with defaults
_raw_spec = oxr.spec
_name = _raw_spec.name
_provider = _raw_spec.provider
_region = _raw_spec.region or regions.DEFAULT_GCP_REGION
_cidr = _raw_spec.cidrBlock or m.DEFAULT_VPC_CIDR
_env = _raw_spec.environment or "dev"
_deletion_policy = _raw_spec.deletionPolicy or "Delete"
_dns_support = _raw_spec.enableDnsSupport if _raw_spec.enableDnsSupport != None else True
_dns_hostnames = _raw_spec.enableDnsHostnames if _raw_spec.enableDnsHostnames != None else True

# Get provider-specific region
_gcp_region = regions.get_gcp_region(_region)
_aws_region = regions.get_aws_region(_region)

# Default subnets if not provided
_subnets = _raw_spec.subnets or m.get_default_subnets(_provider)

# ============================================================================
# GCP Resources
# ============================================================================

_gcp_network = {
    apiVersion = "compute.gcp.upbound.io/v1beta1"
    kind = "Network"
    metadata = h.metadata("${_name}-network")
    spec = {
        forProvider = {
            autoCreateSubnetworks = False
            routingMode = "REGIONAL"
            description = "Managed by Crossplane - ${_name}"
        }
        deletionPolicy = _deletion_policy
    }
}

_gcp_subnetworks = [
    {
        apiVersion = "compute.gcp.upbound.io/v1beta1"
        kind = "Subnetwork"
        metadata = h.metadata("${_name}-${subnet.name}")
        spec = {
            forProvider = {
                region = _gcp_region
                ipCidrRange = subnet.cidrBlock
                networkRef = {
                    name = "${_name}-network"
                }
                privateIpGoogleAccess = subnet.type == "private"
                description = "${subnet.type} subnet - ${subnet.name}"
            }
            deletionPolicy = _deletion_policy
        }
    }
    for subnet in _subnets
]

# ============================================================================
# AWS Resources
# ============================================================================

_aws_vpc = {
    apiVersion = "ec2.aws.upbound.io/v1beta1"
    kind = "VPC"
    metadata = h.metadata("${_name}-vpc")
    spec = {
        forProvider = {
            region = _aws_region
            cidrBlock = _cidr
            enableDnsSupport = _dns_support
            enableDnsHostnames = _dns_hostnames
            tags = h.aws_tags(_env, {"Name": "${_name}-vpc", "network": _name})
        }
        deletionPolicy = _deletion_policy
    }
}

_aws_internet_gateway = {
    apiVersion = "ec2.aws.upbound.io/v1beta1"
    kind = "InternetGateway"
    metadata = h.metadata("${_name}-igw")
    spec = {
        forProvider = {
            region = _aws_region
            vpcIdRef = {
                name = "${_name}-vpc"
            }
            tags = h.aws_tags(_env, {"Name": "${_name}-igw", "network": _name})
        }
        deletionPolicy = _deletion_policy
    }
}

_aws_subnets = [
    {
        apiVersion = "ec2.aws.upbound.io/v1beta1"
        kind = "Subnet"
        metadata = h.metadata("${_name}-${subnet.name}")
        spec = {
            forProvider = {
                region = _aws_region
                cidrBlock = subnet.cidrBlock
                vpcIdRef = {
                    name = "${_name}-vpc"
                }
                availabilityZone = "${_aws_region}${subnet.availabilityZone}" if subnet.availabilityZone else None
                mapPublicIpOnLaunch = subnet.type == "public"
                tags = h.aws_tags(_env, {"Name": "${_name}-${subnet.name}", "Type": subnet.type, "network": _name})
            }
            deletionPolicy = _deletion_policy
        }
    }
    for subnet in _subnets
]

# Public route table for internet access
_aws_public_route_table = {
    apiVersion = "ec2.aws.upbound.io/v1beta1"
    kind = "RouteTable"
    metadata = h.metadata("${_name}-public-rt")
    spec = {
        forProvider = {
            region = _aws_region
            vpcIdRef = {
                name = "${_name}-vpc"
            }
            tags = h.aws_tags(_env, {"Name": "${_name}-public-rt", "network": _name})
        }
        deletionPolicy = _deletion_policy
    }
}

# Route to internet gateway
_aws_public_route = {
    apiVersion = "ec2.aws.upbound.io/v1beta1"
    kind = "Route"
    metadata = h.metadata("${_name}-public-route")
    spec = {
        forProvider = {
            region = _aws_region
            routeTableIdRef = {
                name = "${_name}-public-rt"
            }
            destinationCidrBlock = "0.0.0.0/0"
            gatewayIdRef = {
                name = "${_name}-igw"
            }
        }
        deletionPolicy = _deletion_policy
    }
}

# Route table associations for public subnets
_aws_route_table_associations = [
    {
        apiVersion = "ec2.aws.upbound.io/v1beta1"
        kind = "RouteTableAssociation"
        metadata = h.metadata("${_name}-${subnet.name}-rta")
        spec = {
            forProvider = {
                region = _aws_region
                routeTableIdRef = {
                    name = "${_name}-public-rt"
                }
                subnetIdRef = {
                    name = "${_name}-${subnet.name}"
                }
            }
            deletionPolicy = _deletion_policy
        }
    }
    for subnet in _subnets if subnet.type == "public"
]

# ============================================================================
# Select resources based on provider
# ============================================================================

_gcp_resources = [_gcp_network] + _gcp_subnetworks
_aws_resources = [_aws_vpc, _aws_internet_gateway] + _aws_subnets + [_aws_public_route_table, _aws_public_route] + _aws_route_table_associations

_resources = _aws_resources if _provider == "aws" else _gcp_resources

# Update composite resource status
_status = {
    provider = _provider
    region = _aws_region if _provider == "aws" else _gcp_region
}

_dxr = {
    **oxr
    status = _status
}

# Output items
items = _resources + [_dxr]
