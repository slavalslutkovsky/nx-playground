"""
Crossplane Composition Function Entry Point for Network Infrastructure

This is the main entry point that Crossplane calls when rendering compositions.
It reads the composite resource, extracts parameters, and delegates to the
appropriate provider handlers based on labels.

Supported Providers (via labels):
  - cloud.provider/aws: "true"    -> Creates AWS VPC, subnets, NAT, security groups
  - cloud.provider/gcp: "true"    -> Creates GCP VPC, subnets, Cloud NAT, firewalls
  - cloud.provider/azure: "true"  -> Creates Azure VNet, subnets, NAT Gateway, NSGs
"""

import .main as network

# The main function that Crossplane calls
# oxr = Observed Composite Resource
# ocds = Observed Composed Resources (dictionary)
oxr = option("params")?.oxr or {}
ocds = option("params")?.ocds or {}

# Extract spec parameters from the composite resource
_spec = oxr?.spec or {}
_params = _spec?.parameters or {}
_metadata = oxr?.metadata or {}
_labels = _metadata?.labels or {}
_name = _metadata?.name or "unnamed"

# Determine active providers
_is_aws = _labels.get(network.LABEL_AWS, "false") == "true"
_is_gcp = _labels.get(network.LABEL_GCP, "false") == "true"
_is_azure = _labels.get(network.LABEL_AZURE, "false") == "true"

# Build subnet configuration
_subnet_config = network.SubnetConfig {
    public = network.SubnetTypeConfig {
        count = _params?.subnets?.public?.count or 3
        cidrMask = _params?.subnets?.public?.cidrMask or 24
    }
    private = network.SubnetTypeConfig {
        count = _params?.subnets?.private?.count or 3
        cidrMask = _params?.subnets?.private?.cidrMask or 24
    }
    database = network.DatabaseSubnetConfig {
        enabled = _params?.subnets?.database?.enabled or False
        count = _params?.subnets?.database?.count or 3
        cidrMask = _params?.subnets?.database?.cidrMask or 24
    }
}

# Build connectivity configuration
_connectivity_config = network.ConnectivityConfig {
    natGateway = network.NatGatewayConfig {
        enabled = _params?.connectivity?.natGateway?.enabled if _params?.connectivity?.natGateway else True
        highAvailability = _params?.connectivity?.natGateway?.highAvailability or False
    }
    internetGateway = network.InternetGatewayConfig {
        enabled = _params?.connectivity?.internetGateway?.enabled if _params?.connectivity?.internetGateway else True
    }
    vpcEndpoints = network.VpcEndpointConfig {
        enabled = _params?.connectivity?.vpcEndpoints?.enabled or False
        services = _params?.connectivity?.vpcEndpoints?.services or []
    }
}

# Build security configuration
_security_config = network.SecurityConfig {
    defaultSecurityGroup = network.DefaultSecurityGroupConfig {
        enabled = _params?.security?.defaultSecurityGroup?.enabled if _params?.security?.defaultSecurityGroup else True
        allowInternalTraffic = _params?.security?.defaultSecurityGroup?.allowInternalTraffic if _params?.security?.defaultSecurityGroup else True
        allowOutboundAll = _params?.security?.defaultSecurityGroup?.allowOutboundAll if _params?.security?.defaultSecurityGroup else True
    }
    networkAcls = network.NetworkAclConfig {
        enabled = _params?.security?.networkAcls?.enabled or False
        denyRules = _params?.security?.networkAcls?.denyRules or []
    }
    firewallRules = [
        network.FirewallRule {
            name = rule?.name or "rule"
            priority = rule?.priority or 1000
            direction = rule?.direction or "ingress"
            action = rule?.action or "allow"
            protocol = rule?.protocol or "tcp"
            ports = rule?.ports or []
            sourceCidrs = rule?.sourceCidrs or []
            destinationCidrs = rule?.destinationCidrs or []
        }
        for rule in (_params?.security?.firewallRules or [])
    ]
}

# Build observability configuration
_observability_config = network.ObservabilityConfig {
    flowLogs = network.FlowLogConfig {
        enabled = _params?.observability?.flowLogs?.enabled or False
        trafficType = _params?.observability?.flowLogs?.trafficType or "ALL"
        retentionDays = _params?.observability?.flowLogs?.retentionDays or 14
        destination = _params?.observability?.flowLogs?.destination or "cloud-watch"
    }
    networkWatcher = network.NetworkWatcherConfig {
        enabled = _params?.observability?.networkWatcher?.enabled or False
    }
}

# Build cluster security configuration
_cluster_security_config = network.ClusterSecurityConfig {
    privateClusterEndpoint = _params?.clusterSecurity?.privateClusterEndpoint or False
    podCidrBlock = _params?.clusterSecurity?.podCidrBlock or "10.244.0.0/16"
    serviceCidrBlock = _params?.clusterSecurity?.serviceCidrBlock or "10.96.0.0/12"
    networkPolicy = _params?.clusterSecurity?.networkPolicy if _params?.clusterSecurity else True
}

# Build AWS config if needed
_aws_config = network.AWSConfig {
    availabilityZones = _params?.aws?.availabilityZones or []
    enableVpcPeering = _params?.aws?.enableVpcPeering or False
    transitGateway = network.TransitGatewayConfig {
        enabled = _params?.aws?.transitGateway?.enabled or False
        transitGatewayId = _params?.aws?.transitGateway?.transitGatewayId or ""
    }
} if _is_aws else None

# Build GCP config if needed
_gcp_config = network.GCPConfig {
    project = _params?.gcp?.project or ""
    routingMode = _params?.gcp?.routingMode or "REGIONAL"
    privateGoogleAccess = _params?.gcp?.privateGoogleAccess if _params?.gcp else True
    sharedVpc = network.SharedVpcConfig {
        enabled = _params?.gcp?.sharedVpc?.enabled or False
        hostProject = _params?.gcp?.sharedVpc?.hostProject or ""
    }
} if _is_gcp else None

# Build Azure config if needed
_azure_config = network.AzureConfig {
    resourceGroup = _params?.azure?.resourceGroup or "crossplane-rg"
    addressSpaces = _params?.azure?.addressSpaces or []
    ddosProtection = network.DdosProtectionConfig {
        enabled = _params?.azure?.ddosProtection?.enabled or False
        planId = _params?.azure?.ddosProtection?.planId or ""
    }
    bastionHost = network.BastionHostConfig {
        enabled = _params?.azure?.bastionHost?.enabled or False
        sku = _params?.azure?.bastionHost?.sku or "Basic"
    }
    privateLink = network.PrivateLinkConfig {
        enabled = _params?.azure?.privateLink?.enabled or False
        services = _params?.azure?.privateLink?.services or []
    }
} if _is_azure else None

# Build network parameters from composite resource
networkParams = network.NetworkParams {
    name = _name
    region = _params?.region or "us-east-1"
    cidrBlock = _params?.cidrBlock or "10.0.0.0/16"
    enableDnsSupport = _params?.enableDnsSupport if _params?.enableDnsSupport != None else True
    enableDnsHostnames = _params?.enableDnsHostnames if _params?.enableDnsHostnames != None else True
    tags = _params?.tags or {}
    subnets = _subnet_config
    connectivity = _connectivity_config
    security = _security_config
    observability = _observability_config
    clusterSecurity = _cluster_security_config
    aws = _aws_config
    gcp = _gcp_config
    azure = _azure_config
}

# Create resources based on provider labels
_desired_resources = network.createNetwork(oxr, ocds, networkParams)

# Output desired composed resources
# Format expected by Crossplane function-kcl
items = [
    {
        apiVersion = r.apiVersion
        kind = r.kind
        metadata = r.metadata
        spec = r.spec
    }
    for r in _desired_resources if r
]
