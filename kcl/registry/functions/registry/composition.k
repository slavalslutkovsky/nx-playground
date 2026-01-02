"""
Crossplane Composition Function Entry Point for Container Registry

This is the main entry point that Crossplane calls when rendering compositions.
It reads the composite resource, extracts parameters, and delegates to the
appropriate provider handlers based on labels.

Supported Providers (via labels):
  - cloud.provider/aws: "true"    -> Creates AWS ECR repository
  - cloud.provider/gcp: "true"    -> Creates GCP Artifact Registry repository
  - cloud.provider/azure: "true"  -> Creates Azure Container Registry
"""

import .main as registry

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
_is_aws = _labels.get(registry.LABEL_AWS, "false") == "true"
_is_gcp = _labels.get(registry.LABEL_GCP, "false") == "true"
_is_azure = _labels.get(registry.LABEL_AZURE, "false") == "true"

# Build security configuration
_security_config = registry.SecurityConfig {
    scanOnPush = _params?.security?.scanOnPush if _params?.security else True
    encryptionType = _params?.security?.encryptionType or "AES256"
    kmsKeyId = _params?.security?.kmsKeyId or ""
    publicAccess = _params?.security?.publicAccess or False
}

# Build lifecycle configuration
_lifecycle_config = registry.LifecycleConfig {
    enabled = _params?.lifecycle?.enabled if _params?.lifecycle else True
    maxImageAge = _params?.lifecycle?.maxImageAge or 14
    maxImageCount = _params?.lifecycle?.maxImageCount or 100
    keepTagPatterns = _params?.lifecycle?.keepTagPatterns or ["latest", "v*"]
}

# Build replication configuration
_replication_config = registry.ReplicationConfig {
    enabled = _params?.replication?.enabled or False
    destinationRegions = _params?.replication?.destinationRegions or []
}

# Build access control configuration
_access_control_config = registry.AccessControlConfig {
    pullPrincipals = _params?.accessControl?.pullPrincipals or []
    pushPrincipals = _params?.accessControl?.pushPrincipals or []
    adminPrincipals = _params?.accessControl?.adminPrincipals or []
}

# Build AWS config if needed
_aws_config = registry.AWSConfig {
    forceDelete = _params?.aws?.forceDelete or False
    scanType = _params?.aws?.scanType or "BASIC"
    pullThroughCache = registry.PullThroughCacheConfig {
        enabled = _params?.aws?.pullThroughCache?.enabled or False
        upstreamRegistries = [
            registry.UpstreamRegistry {
                prefix = r?.prefix or ""
                upstream = r?.upstream or "docker.io"
            }
            for r in (_params?.aws?.pullThroughCache?.upstreamRegistries or [])
        ]
    }
} if _is_aws else None

# Build GCP config if needed
_gcp_config = registry.GCPConfig {
    project = _params?.gcp?.project or ""
    format = _params?.gcp?.format or "DOCKER"
    mode = _params?.gcp?.mode or "STANDARD_REPOSITORY"
    cleanupPolicies = registry.CleanupPoliciesConfig {
        enabled = _params?.gcp?.cleanupPolicies?.enabled if _params?.gcp?.cleanupPolicies else True
        dryRun = _params?.gcp?.cleanupPolicies?.dryRun or False
    }
} if _is_gcp else None

# Build Azure config if needed
_azure_config = registry.AzureConfig {
    resourceGroup = _params?.azure?.resourceGroup or "crossplane-rg"
    sku = _params?.azure?.sku or "Standard"
    adminUserEnabled = _params?.azure?.adminUserEnabled or False
    zoneRedundancy = _params?.azure?.zoneRedundancy or False
    networkRules = registry.NetworkRulesConfig {
        defaultAction = _params?.azure?.networkRules?.defaultAction or "Allow"
        ipRules = _params?.azure?.networkRules?.ipRules or []
        virtualNetworkRules = _params?.azure?.networkRules?.virtualNetworkRules or []
    }
    retentionPolicy = registry.RetentionPolicyConfig {
        enabled = _params?.azure?.retentionPolicy?.enabled if _params?.azure?.retentionPolicy else True
        days = _params?.azure?.retentionPolicy?.days or 7
    }
    trustPolicy = registry.TrustPolicyConfig {
        enabled = _params?.azure?.trustPolicy?.enabled or False
    }
} if _is_azure else None

# Build registry parameters from composite resource
registryParams = registry.RegistryParams {
    name = _name
    region = _params?.region or "us-east-1"
    repositoryName = _params?.repositoryName or _name
    imageTagMutability = _params?.imageTagMutability or "MUTABLE"
    tags = _params?.tags or {}
    security = _security_config
    lifecycle = _lifecycle_config
    replication = _replication_config
    accessControl = _access_control_config
    aws = _aws_config
    gcp = _gcp_config
    azure = _azure_config
}

# Create resources based on provider labels
_desired_resources = registry.createContainerRegistry(oxr, ocds, registryParams)

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
