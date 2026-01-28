"""
AWS VPC resource generation for network composition.
"""

import schemas.network as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.regions as regions
import schemas.mappings as m

generate = lambda spec: schema.NetworkSpec, ocds: any -> [any] {
    """Generate AWS VPC resources from NetworkSpec."""
    _name = spec.name
    _region = regions.get_aws_region(spec.region) if spec.region else ""
    _cidr = spec.cidrBlock

    _label_config = labels.LabelConfig {
        name = _name
        component = "network"
        provider = "aws"
        environment = spec.environment
    }
    _tags = labels.generate_tags(_label_config, "aws")
    _k8s_labels = labels.generate_labels(_label_config)

    # Get subnets (use defaults if not provided)
    _subnets = spec.subnets or m.get_default_subnets("aws")

    resources: [any] = []

    # AWS VPC
    resources += [{
        apiVersion = "ec2.aws.upbound.io/v1beta1"
        kind = "VPC"
        metadata = h.metadata_with_labels("${_name}-vpc", _k8s_labels)
        spec = {
            forProvider = {
                region = _region
                cidrBlock = _cidr
                enableDnsSupport = spec.enableDnsSupport
                enableDnsHostnames = spec.enableDnsHostnames
                tags = _tags | {"Name": "${_name}-vpc", "network": _name}
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # Internet Gateway
    resources += [{
        apiVersion = "ec2.aws.upbound.io/v1beta1"
        kind = "InternetGateway"
        metadata = h.metadata_with_labels("${_name}-igw", _k8s_labels)
        spec = {
            forProvider = {
                region = _region
                vpcIdRef = {
                    name = "${_name}-vpc"
                }
                tags = _tags | {"Name": "${_name}-igw", "network": _name}
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # Subnets
    resources += [
        {
            apiVersion = "ec2.aws.upbound.io/v1beta1"
            kind = "Subnet"
            metadata = h.metadata_with_labels("${_name}-${subnet.name}", _k8s_labels | {
                "crossplane.io/network" = _name
                "crossplane.io/visibility" = subnet.type
            })
            spec = {
                forProvider = {
                    region = _region
                    cidrBlock = subnet.cidrBlock
                    vpcIdRef = {
                        name = "${_name}-vpc"
                    }
                    availabilityZone = "${_region}${subnet.availabilityZone}" if subnet.availabilityZone else None
                    mapPublicIpOnLaunch = subnet.type == "public"
                    tags = _tags | {"Name": "${_name}-${subnet.name}", "Type": subnet.type, "network": _name}
                }
                deletionPolicy = spec.deletionPolicy
            }
        }
        for subnet in _subnets
    ]

    # Public Route Table
    resources += [{
        apiVersion = "ec2.aws.upbound.io/v1beta1"
        kind = "RouteTable"
        metadata = h.metadata_with_labels("${_name}-public-rt", _k8s_labels)
        spec = {
            forProvider = {
                region = _region
                vpcIdRef = {
                    name = "${_name}-vpc"
                }
                tags = _tags | {"Name": "${_name}-public-rt", "network": _name}
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # Route to Internet Gateway
    resources += [{
        apiVersion = "ec2.aws.upbound.io/v1beta1"
        kind = "Route"
        metadata = h.metadata_with_labels("${_name}-public-route", _k8s_labels)
        spec = {
            forProvider = {
                region = _region
                routeTableIdRef = {
                    name = "${_name}-public-rt"
                }
                destinationCidrBlock = "0.0.0.0/0"
                gatewayIdRef = {
                    name = "${_name}-igw"
                }
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # Route Table Associations for public subnets
    resources += [
        {
            apiVersion = "ec2.aws.upbound.io/v1beta1"
            kind = "RouteTableAssociation"
            metadata = h.metadata_with_labels("${_name}-${subnet.name}-rta", _k8s_labels)
            spec = {
                forProvider = {
                    region = _region
                    routeTableIdRef = {
                        name = "${_name}-public-rt"
                    }
                    subnetIdRef = {
                        name = "${_name}-${subnet.name}"
                    }
                }
                deletionPolicy = spec.deletionPolicy
            }
        }
        for subnet in _subnets if subnet.type == "public"
    ]

    resources
}

get_status = lambda spec: schema.NetworkSpec, ocds: any -> schema.NetworkStatus {
    """Extract status from observed composed resources."""
    _observed_vpc = ocds.get("${spec.name}-vpc", {})
    _observed_status = _observed_vpc.get("Resource", {}).get("status", {}).get("atProvider", {})

    _vpc_id = _observed_status.get("id", "")

    schema.NetworkStatus {
        provider = "aws"
        region = spec.region
        networkId = _vpc_id
        cidrBlock = spec.cidrBlock
    }
}
