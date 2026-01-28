"""
GCP Network resource generation for network composition.
"""

import schemas.network as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.regions as regions
import schemas.mappings as m

generate = lambda spec: schema.NetworkSpec, ocds: any -> [any] {
    """Generate GCP Network resources from NetworkSpec."""
    _name = spec.name
    _region = regions.get_gcp_region(spec.region) if spec.region else ""

    _label_config = labels.LabelConfig {
        name = _name
        component = "network"
        provider = "gcp"
        environment = spec.environment
    }
    _k8s_labels = labels.generate_labels(_label_config)

    # Get subnets (use defaults if not provided)
    _subnets = spec.subnets or m.get_default_subnets("gcp")

    resources: [any] = []

    # GCP Network (VPC)
    resources += [{
        apiVersion = "compute.gcp.upbound.io/v1beta1"
        kind = "Network"
        metadata = h.metadata_with_labels("${_name}-network", _k8s_labels)
        spec = {
            forProvider = {
                autoCreateSubnetworks = False
                routingMode = "REGIONAL"
                description = "Managed by Crossplane - ${_name}"
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # Subnetworks
    resources += [
        {
            apiVersion = "compute.gcp.upbound.io/v1beta1"
            kind = "Subnetwork"
            metadata = h.metadata_with_labels("${_name}-${subnet.name}", _k8s_labels | {
                "crossplane.io/network" = _name
                "crossplane.io/visibility" = subnet.type
            })
            spec = {
                forProvider = {
                    region = _region
                    ipCidrRange = subnet.cidrBlock
                    networkRef = {
                        name = "${_name}-network"
                    }
                    privateIpGoogleAccess = subnet.type == "private"
                    description = "${subnet.type} subnet - ${subnet.name}"
                }
                deletionPolicy = spec.deletionPolicy
            }
        }
        for subnet in _subnets
    ]

    # Cloud Router for NAT (if there are private subnets)
    _has_private_subnets = any(subnet.type == "private" for subnet in _subnets)
    if _has_private_subnets:
        resources += [{
            apiVersion = "compute.gcp.upbound.io/v1beta1"
            kind = "Router"
            metadata = h.metadata_with_labels("${_name}-router", _k8s_labels)
            spec = {
                forProvider = {
                    region = _region
                    networkRef = {
                        name = "${_name}-network"
                    }
                    description = "NAT router for ${_name}"
                }
                deletionPolicy = spec.deletionPolicy
            }
        }]

        # Cloud NAT for private subnet internet access
        resources += [{
            apiVersion = "compute.gcp.upbound.io/v1beta1"
            kind = "RouterNAT"
            metadata = h.metadata_with_labels("${_name}-nat", _k8s_labels)
            spec = {
                forProvider = {
                    region = _region
                    routerRef = {
                        name = "${_name}-router"
                    }
                    natIpAllocateOption = "AUTO_ONLY"
                    sourceSubnetworkIpRangesToNat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
                }
                deletionPolicy = spec.deletionPolicy
            }
        }]

    resources
}

get_status = lambda spec: schema.NetworkSpec, ocds: any -> schema.NetworkStatus {
    """Extract status from observed composed resources."""
    _observed_network = ocds.get("${spec.name}-network", {})
    _observed_status = _observed_network.get("Resource", {}).get("status", {}).get("atProvider", {})

    _network_id = _observed_status.get("id", "")

    schema.NetworkStatus {
        provider = "gcp"
        region = spec.region
        networkId = _network_id
        cidrBlock = spec.cidrBlock
    }
}
