"""
Unified Network Infrastructure Function for Multi-Cloud Management

This module provides a single entry point for creating network infrastructure across
AWS VPC, GCP VPC, and Azure VNet. Provider selection is handled via labels on the
composite resource.

Features:
  - VPC/VNet creation with custom CIDR blocks
  - Public, private, and database subnets
  - Security groups, firewalls, and network ACLs
  - NAT gateways with optional high availability
  - Internet gateways and route tables
  - VPC Flow Logs for observability
  - Private endpoints for secure cloud service access
  - Cluster security configurations for Kubernetes

Labels:
  - cloud.provider/aws: "true"    -> Creates AWS VPC resources
  - cloud.provider/gcp: "true"    -> Creates GCP VPC resources
  - cloud.provider/azure: "true"  -> Creates Azure VNet resources
"""

# Import typed schemas from generated Upbound models
# AWS VPC resources
import models.io.upbound.awsm.ec2.v1beta1 as ec2
import models.io.upbound.awsm.ec2.v1beta2 as ec2v2

# GCP Compute resources
import models.io.upbound.gcpm.compute.v1beta1 as gcpcompute
import models.io.upbound.gcpm.compute.v1beta2 as gcpcomputev2

# Azure Network resources
import models.io.upbound.azure.network.v1beta1 as azurenet
import models.io.upbound.azure.network.v1beta2 as azurenetv2

# Common Kubernetes metadata
import models.k8s.apimachinery.pkg.apis.meta.v1 as metav1

# ============================================================================
# Provider Label Constants
# ============================================================================

LABEL_AWS = "cloud.provider/aws"
LABEL_GCP = "cloud.provider/gcp"
LABEL_AZURE = "cloud.provider/azure"

# ============================================================================
# Schema Definitions
# ============================================================================

schema NetworkParams:
    """Common parameters for all cloud network providers"""
    name: str
    region: str
    cidrBlock: str = "10.0.0.0/16"
    enableDnsSupport: bool = True
    enableDnsHostnames: bool = True
    tags: {str: str} = {}

    # Subnet configuration
    subnets: SubnetConfig = SubnetConfig {}

    # Connectivity configuration
    connectivity: ConnectivityConfig = ConnectivityConfig {}

    # Security configuration
    security: SecurityConfig = SecurityConfig {}

    # Observability configuration
    observability: ObservabilityConfig = ObservabilityConfig {}

    # Cluster security configuration
    clusterSecurity: ClusterSecurityConfig = ClusterSecurityConfig {}

    # Provider-specific configs
    aws: AWSConfig = None
    gcp: GCPConfig = None
    azure: AzureConfig = None

    check:
        len(name) >= 1 and len(name) <= 63, "Network name must be 1-63 characters"
        len(cidrBlock) >= 9, "CIDR block must be valid (e.g., 10.0.0.0/16)"

schema SubnetConfig:
    """Subnet configuration for all subnet types"""
    public: SubnetTypeConfig = SubnetTypeConfig {count = 3, cidrMask = 24}
    private: SubnetTypeConfig = SubnetTypeConfig {count = 3, cidrMask = 24}
    database: DatabaseSubnetConfig = DatabaseSubnetConfig {}

schema SubnetTypeConfig:
    """Configuration for a subnet type"""
    count: int = 3
    cidrMask: int = 24

    check:
        count >= 1 and count <= 6, "Subnet count must be 1-6"
        cidrMask >= 16 and cidrMask <= 28, "CIDR mask must be 16-28"

schema DatabaseSubnetConfig:
    """Database subnet configuration"""
    enabled: bool = False
    count: int = 3
    cidrMask: int = 24

schema ConnectivityConfig:
    """Network connectivity options"""
    natGateway: NatGatewayConfig = NatGatewayConfig {}
    internetGateway: InternetGatewayConfig = InternetGatewayConfig {}
    vpcEndpoints: VpcEndpointConfig = VpcEndpointConfig {}

schema NatGatewayConfig:
    """NAT Gateway configuration"""
    enabled: bool = True
    highAvailability: bool = False

schema InternetGatewayConfig:
    """Internet Gateway configuration"""
    enabled: bool = True

schema VpcEndpointConfig:
    """VPC Endpoint configuration"""
    enabled: bool = False
    services: [str] = []

schema SecurityConfig:
    """Network security configuration"""
    defaultSecurityGroup: DefaultSecurityGroupConfig = DefaultSecurityGroupConfig {}
    networkAcls: NetworkAclConfig = NetworkAclConfig {}
    firewallRules: [FirewallRule] = []

schema DefaultSecurityGroupConfig:
    """Default security group settings"""
    enabled: bool = True
    allowInternalTraffic: bool = True
    allowOutboundAll: bool = True

schema NetworkAclConfig:
    """Network ACL configuration"""
    enabled: bool = False
    denyRules: [str] = []

schema FirewallRule:
    """Firewall rule definition"""
    name: str
    priority: int = 1000
    direction: str = "ingress"
    action: str = "allow"
    protocol: str = "tcp"
    ports: [str] = []
    sourceCidrs: [str] = []
    destinationCidrs: [str] = []

    check:
        direction in ["ingress", "egress"], "Direction must be ingress or egress"
        action in ["allow", "deny"], "Action must be allow or deny"

schema ObservabilityConfig:
    """Observability and monitoring configuration"""
    flowLogs: FlowLogConfig = FlowLogConfig {}
    networkWatcher: NetworkWatcherConfig = NetworkWatcherConfig {}

schema FlowLogConfig:
    """VPC Flow Log configuration"""
    enabled: bool = False
    trafficType: str = "ALL"
    retentionDays: int = 14
    destination: str = "cloud-watch"

    check:
        trafficType in ["ALL", "ACCEPT", "REJECT"], "Traffic type must be ALL, ACCEPT, or REJECT"

schema NetworkWatcherConfig:
    """Network monitoring service configuration"""
    enabled: bool = False

schema ClusterSecurityConfig:
    """Kubernetes cluster network security settings"""
    privateClusterEndpoint: bool = False
    podCidrBlock: str = "10.244.0.0/16"
    serviceCidrBlock: str = "10.96.0.0/12"
    networkPolicy: bool = True

schema AWSConfig:
    """AWS VPC specific configuration"""
    availabilityZones: [str] = []
    enableVpcPeering: bool = False
    transitGateway: TransitGatewayConfig = TransitGatewayConfig {}

schema TransitGatewayConfig:
    """Transit Gateway configuration"""
    enabled: bool = False
    transitGatewayId: str = ""

schema GCPConfig:
    """GCP VPC specific configuration"""
    project: str = ""
    routingMode: str = "REGIONAL"
    privateGoogleAccess: bool = True
    sharedVpc: SharedVpcConfig = SharedVpcConfig {}

    check:
        routingMode in ["REGIONAL", "GLOBAL"], "Routing mode must be REGIONAL or GLOBAL"

schema SharedVpcConfig:
    """Shared VPC configuration"""
    enabled: bool = False
    hostProject: str = ""

schema AzureConfig:
    """Azure VNet specific configuration"""
    resourceGroup: str
    addressSpaces: [str] = []
    ddosProtection: DdosProtectionConfig = DdosProtectionConfig {}
    bastionHost: BastionHostConfig = BastionHostConfig {}
    privateLink: PrivateLinkConfig = PrivateLinkConfig {}

schema DdosProtectionConfig:
    """DDoS Protection configuration"""
    enabled: bool = False
    planId: str = ""

schema BastionHostConfig:
    """Azure Bastion configuration"""
    enabled: bool = False
    sku: str = "Basic"

    check:
        sku in ["Basic", "Standard"], "Bastion SKU must be Basic or Standard"

schema PrivateLinkConfig:
    """Private Link configuration"""
    enabled: bool = False
    services: [str] = []

# ============================================================================
# Utility Functions
# ============================================================================

calculateSubnetCidr = lambda baseIndex: int, subnetIndex: int, cidrMask: int -> str {
    """Calculate subnet CIDR from base network"""
    # Simple calculation: increment the third octet for each subnet
    _third_octet = baseIndex * 16 + subnetIndex
    "10.0.${_third_octet}.0/${cidrMask}"
}

getDefaultAzs = lambda region: str, count: int -> [str] {
    """Get default availability zones for a region"""
    [
        "${region}${suffix}"
        for suffix in ["a", "b", "c", "d", "e", "f"][:count]
    ]
}

# ============================================================================
# Main Composition Function
# ============================================================================

createNetwork = lambda oxr: any, ocds: any, params: NetworkParams -> [any] {
    """
    Creates network infrastructure based on provider labels.
    Returns a list of Crossplane managed resources.
    """
    labels = oxr?.metadata?.labels or {}

    # Determine which providers to create resources for
    _is_aws = labels.get(LABEL_AWS, "false") == "true"
    _is_gcp = labels.get(LABEL_GCP, "false") == "true"
    _is_azure = labels.get(LABEL_AZURE, "false") == "true"

    # Build resources based on labels
    _aws_resources = createAWSResources(params) if _is_aws else []
    _gcp_resources = createGCPResources(params) if _is_gcp else []
    _azure_resources = createAzureResources(params) if _is_azure else []

    _aws_resources + _gcp_resources + _azure_resources
}

# ============================================================================
# AWS VPC Resource Creation
# ============================================================================

createAWSResources = lambda params: NetworkParams -> [any] {
    """Creates AWS VPC and related resources"""
    _name = "${params.name}-aws"
    _region = params.aws?.region if params.aws else params.region
    _azs = params.aws?.availabilityZones if params.aws?.availabilityZones else getDefaultAzs(_region, params.subnets.public.count)
    _merged_tags = {k: v for k, v in params.tags} | {"ManagedBy": "crossplane", "Provider": "aws"}

    # Core VPC
    _vpc = ec2.VPC {
        metadata = metav1.ObjectMeta {
            name = _name
            labels = {
                "${LABEL_AWS}" = "true"
                "network.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                region = _region
                cidrBlock = params.cidrBlock
                enableDnsSupport = params.enableDnsSupport
                enableDnsHostnames = params.enableDnsHostnames
                tags = _merged_tags | {"Name": _name}
            }
        }
    }

    # Internet Gateway
    _igw = ec2.InternetGateway {
        metadata = metav1.ObjectMeta {
            name = "${_name}-igw"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                vpcIdRef = {name = _name}
                tags = _merged_tags | {"Name": "${_name}-igw"}
            }
        }
    } if params.connectivity.internetGateway.enabled else None

    # Public Subnets
    _public_subnets = [
        ec2.Subnet {
            metadata = metav1.ObjectMeta {
                name = "${_name}-public-${i + 1}"
                labels = {
                    "${LABEL_AWS}": "true"
                    "subnet-type": "public"
                }
            }
            spec = {
                forProvider = {
                    region = _region
                    vpcIdRef = {name = _name}
                    cidrBlock = calculateSubnetCidr(0, i, params.subnets.public.cidrMask)
                    availabilityZone = _azs[i] if i < len(_azs) else _azs[0]
                    mapPublicIpOnLaunch = True
                    tags = _merged_tags | {
                        "Name": "${_name}-public-${i + 1}"
                        "kubernetes.io/role/elb": "1"
                        "Type": "public"
                    }
                }
            }
        }
        for i in range(params.subnets.public.count)
    ]

    # Private Subnets
    _private_subnets = [
        ec2.Subnet {
            metadata = metav1.ObjectMeta {
                name = "${_name}-private-${i + 1}"
                labels = {
                    "${LABEL_AWS}": "true"
                    "subnet-type": "private"
                }
            }
            spec = {
                forProvider = {
                    region = _region
                    vpcIdRef = {name = _name}
                    cidrBlock = calculateSubnetCidr(1, i, params.subnets.private.cidrMask)
                    availabilityZone = _azs[i] if i < len(_azs) else _azs[0]
                    mapPublicIpOnLaunch = False
                    tags = _merged_tags | {
                        "Name": "${_name}-private-${i + 1}"
                        "kubernetes.io/role/internal-elb": "1"
                        "Type": "private"
                    }
                }
            }
        }
        for i in range(params.subnets.private.count)
    ]

    # Database Subnets (optional)
    _db_subnets = [
        ec2.Subnet {
            metadata = metav1.ObjectMeta {
                name = "${_name}-database-${i + 1}"
                labels = {
                    "${LABEL_AWS}": "true"
                    "subnet-type": "database"
                }
            }
            spec = {
                forProvider = {
                    region = _region
                    vpcIdRef = {name = _name}
                    cidrBlock = calculateSubnetCidr(2, i, params.subnets.database.cidrMask)
                    availabilityZone = _azs[i] if i < len(_azs) else _azs[0]
                    mapPublicIpOnLaunch = False
                    tags = _merged_tags | {
                        "Name": "${_name}-database-${i + 1}"
                        "Type": "database"
                    }
                }
            }
        }
        for i in range(params.subnets.database.count)
    ] if params.subnets.database.enabled else []

    # Elastic IPs for NAT Gateways
    _nat_count = params.subnets.public.count if params.connectivity.natGateway.highAvailability else 1
    _nat_eips = [
        ec2.EIP {
            metadata = metav1.ObjectMeta {
                name = "${_name}-nat-eip-${i + 1}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    domain = "vpc"
                    tags = _merged_tags | {"Name": "${_name}-nat-eip-${i + 1}"}
                }
            }
        }
        for i in range(_nat_count)
    ] if params.connectivity.natGateway.enabled else []

    # NAT Gateways
    _nat_gateways = [
        ec2.NATGateway {
            metadata = metav1.ObjectMeta {
                name = "${_name}-nat-${i + 1}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    subnetIdRef = {name = "${_name}-public-${i + 1}"}
                    allocationIdRef = {name = "${_name}-nat-eip-${i + 1}"}
                    tags = _merged_tags | {"Name": "${_name}-nat-${i + 1}"}
                }
            }
        }
        for i in range(_nat_count)
    ] if params.connectivity.natGateway.enabled else []

    # Public Route Table
    _public_rt = ec2.RouteTable {
        metadata = metav1.ObjectMeta {
            name = "${_name}-public-rt"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                vpcIdRef = {name = _name}
                tags = _merged_tags | {"Name": "${_name}-public-rt"}
            }
        }
    }

    # Public Route (to Internet Gateway)
    _public_route = ec2.Route {
        metadata = metav1.ObjectMeta {
            name = "${_name}-public-route"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                routeTableIdRef = {name = "${_name}-public-rt"}
                destinationCidrBlock = "0.0.0.0/0"
                gatewayIdRef = {name = "${_name}-igw"}
            }
        }
    } if params.connectivity.internetGateway.enabled else None

    # Public Route Table Associations
    _public_rt_assocs = [
        ec2.RouteTableAssociation {
            metadata = metav1.ObjectMeta {
                name = "${_name}-public-rta-${i + 1}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    routeTableIdRef = {name = "${_name}-public-rt"}
                    subnetIdRef = {name = "${_name}-public-${i + 1}"}
                }
            }
        }
        for i in range(params.subnets.public.count)
    ]

    # Private Route Tables (one per NAT for HA, or single)
    _private_rts = [
        ec2.RouteTable {
            metadata = metav1.ObjectMeta {
                name = "${_name}-private-rt-${i + 1}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    vpcIdRef = {name = _name}
                    tags = _merged_tags | {"Name": "${_name}-private-rt-${i + 1}"}
                }
            }
        }
        for i in range(_nat_count if params.connectivity.natGateway.highAvailability else 1)
    ]

    # Private Routes (to NAT Gateway)
    _private_routes = [
        ec2.Route {
            metadata = metav1.ObjectMeta {
                name = "${_name}-private-route-${i + 1}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    routeTableIdRef = {name = "${_name}-private-rt-${i + 1}"}
                    destinationCidrBlock = "0.0.0.0/0"
                    natGatewayIdRef = {name = "${_name}-nat-${i + 1}"}
                }
            }
        }
        for i in range(_nat_count if params.connectivity.natGateway.highAvailability else 1)
    ] if params.connectivity.natGateway.enabled else []

    # Private Route Table Associations
    _private_rt_assocs = [
        ec2.RouteTableAssociation {
            metadata = metav1.ObjectMeta {
                name = "${_name}-private-rta-${i + 1}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    routeTableIdRef = {
                        name = if params.connectivity.natGateway.highAvailability then "${_name}-private-rt-${(i % _nat_count) + 1}" else "${_name}-private-rt-1"
                    }
                    subnetIdRef = {name = "${_name}-private-${i + 1}"}
                }
            }
        }
        for i in range(params.subnets.private.count)
    ]

    # Default Security Group
    _default_sg = ec2.SecurityGroup {
        metadata = metav1.ObjectMeta {
            name = "${_name}-default-sg"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                vpcIdRef = {name = _name}
                name = "${_name}-default"
                description = "Default security group for ${_name}"
                tags = _merged_tags | {"Name": "${_name}-default-sg"}
            }
        }
    } if params.security.defaultSecurityGroup.enabled else None

    # Security Group Rules - Internal Traffic
    _sg_internal_ingress = ec2.SecurityGroupRule {
        metadata = metav1.ObjectMeta {
            name = "${_name}-sg-internal-ingress"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                securityGroupIdRef = {name = "${_name}-default-sg"}
                type = "ingress"
                fromPort = 0
                toPort = 65535
                protocol = "-1"
                cidrBlocks = [params.cidrBlock]
                description = "Allow all internal VPC traffic"
            }
        }
    } if params.security.defaultSecurityGroup.enabled and params.security.defaultSecurityGroup.allowInternalTraffic else None

    # Security Group Rules - Outbound All
    _sg_egress_all = ec2.SecurityGroupRule {
        metadata = metav1.ObjectMeta {
            name = "${_name}-sg-egress-all"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                securityGroupIdRef = {name = "${_name}-default-sg"}
                type = "egress"
                fromPort = 0
                toPort = 65535
                protocol = "-1"
                cidrBlocks = ["0.0.0.0/0"]
                description = "Allow all outbound traffic"
            }
        }
    } if params.security.defaultSecurityGroup.enabled and params.security.defaultSecurityGroup.allowOutboundAll else None

    # VPC Flow Logs
    _flow_log = ec2.FlowLog {
        metadata = metav1.ObjectMeta {
            name = "${_name}-flow-log"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                vpcIdRef = {name = _name}
                trafficType = params.observability.flowLogs.trafficType
                logDestinationType = "cloud-watch-logs"
                maxAggregationInterval = 60
                tags = _merged_tags | {"Name": "${_name}-flow-log"}
            }
        }
    } if params.observability.flowLogs.enabled else None

    # Combine all resources
    [r for r in [
        _vpc,
        _igw,
        _public_rt,
        _public_route,
        _default_sg,
        _sg_internal_ingress,
        _sg_egress_all,
        _flow_log,
    ] + _public_subnets + _private_subnets + _db_subnets + _nat_eips + _nat_gateways + _private_rts + _private_routes + _public_rt_assocs + _private_rt_assocs if r]
}

# ============================================================================
# GCP VPC Resource Creation
# ============================================================================

createGCPResources = lambda params: NetworkParams -> [any] {
    """Creates GCP VPC and related resources"""
    _name = "${params.name}-gcp"
    _project = params.gcp?.project if params.gcp else ""
    _routing_mode = params.gcp?.routingMode if params.gcp else "REGIONAL"
    _merged_labels = {k.lower().replace("_", "-"): v.lower() for k, v in params.tags} | {"managed-by": "crossplane", "provider": "gcp"}

    # VPC Network
    _vpc = gcpcompute.Network {
        metadata = metav1.ObjectMeta {
            name = _name
            labels = {
                "${LABEL_GCP}" = "true"
                "network.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                autoCreateSubnetworks = False
                routingMode = _routing_mode
                description = "Managed VPC network for ${params.name}"
            }
        }
    }

    # Public Subnets
    _public_subnets = [
        gcpcompute.Subnetwork {
            metadata = metav1.ObjectMeta {
                name = "${_name}-public-${i + 1}"
                labels = {
                    "${LABEL_GCP}": "true"
                    "subnet-type": "public"
                }
            }
            spec = {
                forProvider = {
                    project = _project if _project else None
                    region = params.region
                    networkRef = {name = _name}
                    ipCidrRange = calculateSubnetCidr(0, i, params.subnets.public.cidrMask)
                    privateIpGoogleAccess = False
                    description = "Public subnet ${i + 1} for ${params.name}"
                    purpose = "PRIVATE"
                }
            }
        }
        for i in range(params.subnets.public.count)
    ]

    # Private Subnets with Private Google Access
    _private_subnets = [
        gcpcompute.Subnetwork {
            metadata = metav1.ObjectMeta {
                name = "${_name}-private-${i + 1}"
                labels = {
                    "${LABEL_GCP}": "true"
                    "subnet-type": "private"
                }
            }
            spec = {
                forProvider = {
                    project = _project if _project else None
                    region = params.region
                    networkRef = {name = _name}
                    ipCidrRange = calculateSubnetCidr(1, i, params.subnets.private.cidrMask)
                    privateIpGoogleAccess = params.gcp?.privateGoogleAccess if params.gcp else True
                    description = "Private subnet ${i + 1} for ${params.name}"
                    purpose = "PRIVATE"
                    # Secondary ranges for GKE pods/services
                    secondaryIpRange = [
                        {
                            rangeName = "pods"
                            ipCidrRange = "10.${100 + i}.0.0/16"
                        }
                        {
                            rangeName = "services"
                            ipCidrRange = "10.${200 + i}.0.0/20"
                        }
                    ] if params.clusterSecurity.networkPolicy else None
                }
            }
        }
        for i in range(params.subnets.private.count)
    ]

    # Cloud Router for NAT
    _router = gcpcompute.Router {
        metadata = metav1.ObjectMeta {
            name = "${_name}-router"
            labels = {"${LABEL_GCP}": "true"}
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                region = params.region
                networkRef = {name = _name}
                description = "Cloud Router for ${params.name} NAT"
            }
        }
    } if params.connectivity.natGateway.enabled else None

    # Cloud NAT
    _nat = gcpcompute.RouterNAT {
        metadata = metav1.ObjectMeta {
            name = "${_name}-nat"
            labels = {"${LABEL_GCP}": "true"}
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                region = params.region
                routerRef = {name = "${_name}-router"}
                natIpAllocateOption = "AUTO_ONLY"
                sourceSubnetworkIpRangesToNat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
                logConfig = {
                    enable = params.observability.flowLogs.enabled
                    filter = "ALL" if params.observability.flowLogs.trafficType == "ALL" else "ERRORS_ONLY"
                } if params.observability.flowLogs.enabled else None
            }
        }
    } if params.connectivity.natGateway.enabled else None

    # Firewall Rules - Allow Internal
    _fw_internal = gcpcompute.Firewall {
        metadata = metav1.ObjectMeta {
            name = "${_name}-allow-internal"
            labels = {"${LABEL_GCP}": "true"}
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                networkRef = {name = _name}
                direction = "INGRESS"
                priority = 1000
                sourceRanges = [params.cidrBlock]
                allow = [
                    {protocol = "tcp", ports = ["0-65535"]}
                    {protocol = "udp", ports = ["0-65535"]}
                    {protocol = "icmp"}
                ]
                description = "Allow all internal VPC traffic"
            }
        }
    } if params.security.defaultSecurityGroup.enabled and params.security.defaultSecurityGroup.allowInternalTraffic else None

    # Firewall Rules - Allow Egress
    _fw_egress = gcpcompute.Firewall {
        metadata = metav1.ObjectMeta {
            name = "${_name}-allow-egress"
            labels = {"${LABEL_GCP}": "true"}
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                networkRef = {name = _name}
                direction = "EGRESS"
                priority = 1000
                destinationRanges = ["0.0.0.0/0"]
                allow = [{protocol = "all"}]
                description = "Allow all outbound traffic"
            }
        }
    } if params.security.defaultSecurityGroup.enabled and params.security.defaultSecurityGroup.allowOutboundAll else None

    # Firewall Rules - Deny SSH from Internet (security best practice)
    _fw_deny_ssh = gcpcompute.Firewall {
        metadata = metav1.ObjectMeta {
            name = "${_name}-deny-ssh-internet"
            labels = {"${LABEL_GCP}": "true"}
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                networkRef = {name = _name}
                direction = "INGRESS"
                priority = 500
                sourceRanges = ["0.0.0.0/0"]
                deny = [{protocol = "tcp", ports = ["22"]}]
                description = "Deny SSH from internet (use IAP instead)"
            }
        }
    }

    # Firewall Rules - Allow IAP for secure access
    _fw_allow_iap = gcpcompute.Firewall {
        metadata = metav1.ObjectMeta {
            name = "${_name}-allow-iap"
            labels = {"${LABEL_GCP}": "true"}
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                networkRef = {name = _name}
                direction = "INGRESS"
                priority = 1000
                sourceRanges = ["35.235.240.0/20"]  # IAP IP range
                allow = [
                    {protocol = "tcp", ports = ["22", "3389"]}
                ]
                description = "Allow SSH/RDP via Identity-Aware Proxy"
            }
        }
    }

    # Custom Firewall Rules
    _custom_fw_rules = [
        gcpcompute.Firewall {
            metadata = metav1.ObjectMeta {
                name = "${_name}-${rule.name}"
                labels = {"${LABEL_GCP}": "true"}
            }
            spec = {
                forProvider = {
                    project = _project if _project else None
                    networkRef = {name = _name}
                    direction = "INGRESS" if rule.direction == "ingress" else "EGRESS"
                    priority = rule.priority
                    sourceRanges = rule.sourceCidrs if rule.direction == "ingress" else None
                    destinationRanges = rule.destinationCidrs if rule.direction == "egress" else None
                    allow = [{protocol = rule.protocol, ports = rule.ports}] if rule.action == "allow" else None
                    deny = [{protocol = rule.protocol, ports = rule.ports}] if rule.action == "deny" else None
                    description = "Custom firewall rule: ${rule.name}"
                }
            }
        }
        for rule in params.security.firewallRules
    ]

    # Combine all resources
    [r for r in [
        _vpc,
        _router,
        _nat,
        _fw_internal,
        _fw_egress,
        _fw_deny_ssh,
        _fw_allow_iap,
    ] + _public_subnets + _private_subnets + _custom_fw_rules if r]
}

# ============================================================================
# Azure VNet Resource Creation
# ============================================================================

createAzureResources = lambda params: NetworkParams -> [any] {
    """Creates Azure VNet and related resources"""
    _name = "${params.name}-azure"
    _rg = params.azure?.resourceGroup if params.azure else "default-rg"
    _merged_tags = {k: v for k, v in params.tags} | {"ManagedBy": "crossplane", "Provider": "azure"}

    # Virtual Network
    _vnet = azurenetv2.VirtualNetwork {
        metadata = metav1.ObjectMeta {
            name = _name
            labels = {
                "${LABEL_AZURE}" = "true"
                "network.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                location = params.region
                addressSpace = [params.cidrBlock] + (params.azure?.addressSpaces if params.azure?.addressSpaces else [])
                dnsServers = None  # Use Azure-provided DNS
                tags = _merged_tags
            }
        }
    }

    # Public Subnets
    _public_subnets = [
        azurenetv2.Subnet {
            metadata = metav1.ObjectMeta {
                name = "${_name}-public-${i + 1}"
                labels = {
                    "${LABEL_AZURE}": "true"
                    "subnet-type": "public"
                }
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    virtualNetworkNameRef = {name = _name}
                    addressPrefixes = [calculateSubnetCidr(0, i, params.subnets.public.cidrMask)]
                    serviceEndpoints = ["Microsoft.Storage", "Microsoft.KeyVault"]
                }
            }
        }
        for i in range(params.subnets.public.count)
    ]

    # Private Subnets
    _private_subnets = [
        azurenetv2.Subnet {
            metadata = metav1.ObjectMeta {
                name = "${_name}-private-${i + 1}"
                labels = {
                    "${LABEL_AZURE}": "true"
                    "subnet-type": "private"
                }
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    virtualNetworkNameRef = {name = _name}
                    addressPrefixes = [calculateSubnetCidr(1, i, params.subnets.private.cidrMask)]
                    serviceEndpoints = ["Microsoft.Storage", "Microsoft.Sql", "Microsoft.KeyVault", "Microsoft.ContainerRegistry"]
                    privateEndpointNetworkPolicies = "Disabled"  # Required for Private Endpoints
                }
            }
        }
        for i in range(params.subnets.private.count)
    ]

    # Database Subnets (isolated)
    _db_subnets = [
        azurenetv2.Subnet {
            metadata = metav1.ObjectMeta {
                name = "${_name}-database-${i + 1}"
                labels = {
                    "${LABEL_AZURE}": "true"
                    "subnet-type": "database"
                }
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    virtualNetworkNameRef = {name = _name}
                    addressPrefixes = [calculateSubnetCidr(2, i, params.subnets.database.cidrMask)]
                    serviceEndpoints = ["Microsoft.Sql"]
                    delegation = [
                        {
                            name = "mysql-delegation"
                            serviceDelegation = {
                                name = "Microsoft.DBforMySQL/flexibleServers"
                                actions = ["Microsoft.Network/virtualNetworks/subnets/join/action"]
                            }
                        }
                    ]
                }
            }
        }
        for i in range(params.subnets.database.count)
    ] if params.subnets.database.enabled else []

    # Public IPs for NAT Gateway
    _nat_count = params.subnets.public.count if params.connectivity.natGateway.highAvailability else 1
    _nat_pips = [
        azurenet.PublicIP {
            metadata = metav1.ObjectMeta {
                name = "${_name}-nat-pip-${i + 1}"
                labels = {"${LABEL_AZURE}": "true"}
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    location = params.region
                    allocationMethod = "Static"
                    sku = "Standard"
                    tags = _merged_tags
                }
            }
        }
        for i in range(_nat_count)
    ] if params.connectivity.natGateway.enabled else []

    # NAT Gateways
    _nat_gateways = [
        azurenet.NATGateway {
            metadata = metav1.ObjectMeta {
                name = "${_name}-nat-${i + 1}"
                labels = {"${LABEL_AZURE}": "true"}
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    location = params.region
                    skuName = "Standard"
                    idleTimeoutInMinutes = 10
                    tags = _merged_tags
                }
            }
        }
        for i in range(_nat_count)
    ] if params.connectivity.natGateway.enabled else []

    # NAT Gateway Public IP Associations
    _nat_pip_assocs = [
        azurenet.NATGatewayPublicIPAssociation {
            metadata = metav1.ObjectMeta {
                name = "${_name}-nat-pip-assoc-${i + 1}"
                labels = {"${LABEL_AZURE}": "true"}
            }
            spec = {
                forProvider = {
                    natGatewayIdRef = {name = "${_name}-nat-${i + 1}"}
                    publicIpAddressIdRef = {name = "${_name}-nat-pip-${i + 1}"}
                }
            }
        }
        for i in range(_nat_count)
    ] if params.connectivity.natGateway.enabled else []

    # Network Security Group
    _nsg = azurenetv2.SecurityGroup {
        metadata = metav1.ObjectMeta {
            name = "${_name}-default-nsg"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                location = params.region
                tags = _merged_tags
            }
        }
    } if params.security.defaultSecurityGroup.enabled else None

    # NSG Rules - Allow Internal
    _nsg_internal = azurenet.SecurityRule {
        metadata = metav1.ObjectMeta {
            name = "${_name}-allow-internal"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                networkSecurityGroupNameRef = {name = "${_name}-default-nsg"}
                priority = 100
                direction = "Inbound"
                access = "Allow"
                protocol = "*"
                sourcePortRange = "*"
                destinationPortRange = "*"
                sourceAddressPrefix = params.cidrBlock
                destinationAddressPrefix = params.cidrBlock
                description = "Allow all internal VNet traffic"
            }
        }
    } if params.security.defaultSecurityGroup.enabled and params.security.defaultSecurityGroup.allowInternalTraffic else None

    # NSG Rules - Allow Outbound
    _nsg_outbound = azurenet.SecurityRule {
        metadata = metav1.ObjectMeta {
            name = "${_name}-allow-outbound"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                networkSecurityGroupNameRef = {name = "${_name}-default-nsg"}
                priority = 100
                direction = "Outbound"
                access = "Allow"
                protocol = "*"
                sourcePortRange = "*"
                destinationPortRange = "*"
                sourceAddressPrefix = "*"
                destinationAddressPrefix = "*"
                description = "Allow all outbound traffic"
            }
        }
    } if params.security.defaultSecurityGroup.enabled and params.security.defaultSecurityGroup.allowOutboundAll else None

    # NSG Rules - Deny SSH from Internet (security best practice)
    _nsg_deny_ssh = azurenet.SecurityRule {
        metadata = metav1.ObjectMeta {
            name = "${_name}-deny-ssh-internet"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                networkSecurityGroupNameRef = {name = "${_name}-default-nsg"}
                priority = 200
                direction = "Inbound"
                access = "Deny"
                protocol = "Tcp"
                sourcePortRange = "*"
                destinationPortRange = "22"
                sourceAddressPrefix = "Internet"
                destinationAddressPrefix = "*"
                description = "Deny SSH from internet (use Bastion instead)"
            }
        }
    } if params.security.defaultSecurityGroup.enabled else None

    # NSG Rules - Deny RDP from Internet
    _nsg_deny_rdp = azurenet.SecurityRule {
        metadata = metav1.ObjectMeta {
            name = "${_name}-deny-rdp-internet"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                networkSecurityGroupNameRef = {name = "${_name}-default-nsg"}
                priority = 201
                direction = "Inbound"
                access = "Deny"
                protocol = "Tcp"
                sourcePortRange = "*"
                destinationPortRange = "3389"
                sourceAddressPrefix = "Internet"
                destinationAddressPrefix = "*"
                description = "Deny RDP from internet (use Bastion instead)"
            }
        }
    } if params.security.defaultSecurityGroup.enabled else None

    # Route Table for Private Subnets
    _private_rt = azurenetv2.RouteTable {
        metadata = metav1.ObjectMeta {
            name = "${_name}-private-rt"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                location = params.region
                disableBgpRoutePropagation = False
                tags = _merged_tags
            }
        }
    }

    # Network Watcher for Observability
    _network_watcher = azurenet.Watcher {
        metadata = metav1.ObjectMeta {
            name = "${_name}-watcher"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                location = params.region
                tags = _merged_tags
            }
        }
    } if params.observability.networkWatcher.enabled else None

    # Bastion Host Subnet (required for Azure Bastion)
    _bastion_subnet = azurenetv2.Subnet {
        metadata = metav1.ObjectMeta {
            name = "${_name}-bastion-subnet"
            labels = {
                "${LABEL_AZURE}": "true"
                "subnet-type": "bastion"
            }
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                virtualNetworkNameRef = {name = _name}
                # Bastion requires specific naming and /26 or larger
                name = "AzureBastionSubnet"
                addressPrefixes = ["10.0.255.0/26"]
            }
        }
    } if params.azure?.bastionHost?.enabled else None

    # Bastion Public IP
    _bastion_pip = azurenet.PublicIP {
        metadata = metav1.ObjectMeta {
            name = "${_name}-bastion-pip"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                location = params.region
                allocationMethod = "Static"
                sku = "Standard"
                tags = _merged_tags
            }
        }
    } if params.azure?.bastionHost?.enabled else None

    # Combine all resources
    [r for r in [
        _vnet,
        _private_rt,
        _nsg,
        _nsg_internal,
        _nsg_outbound,
        _nsg_deny_ssh,
        _nsg_deny_rdp,
        _network_watcher,
        _bastion_subnet,
        _bastion_pip,
    ] + _public_subnets + _private_subnets + _db_subnets + _nat_pips + _nat_gateways + _nat_pip_assocs if r]
}

# ============================================================================
# Utility Functions
# ============================================================================

filterByProvider = lambda resources: [any], provider: str -> [any] {
    """Filters composed resources by provider label"""
    _label_key = provider if provider.startswith("cloud.provider/") else "cloud.provider/${provider}"
    [r for r in resources if r?.metadata?.labels?.get(_label_key, "false") == "true"]
}

getActiveProviders = lambda labels: {str: str} -> [str] {
    """Returns list of active providers based on labels"""
    _aws = ["aws"] if (labels.get(LABEL_AWS, "false")) == "true" else []
    _gcp = ["gcp"] if (labels.get(LABEL_GCP, "false")) == "true" else []
    _azure = ["azure"] if (labels.get(LABEL_AZURE, "false")) == "true" else []
    _aws + _gcp + _azure
}

filterBySubnetType = lambda resources: [any], subnetType: str -> [any] {
    """Filters resources by subnet type label"""
    [r for r in resources if r?.metadata?.labels?.get("subnet-type", "") == subnetType]
}
