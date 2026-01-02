"""
Unified Container Registry Function for Multi-Cloud Management

This module provides a single entry point for creating container registries across
AWS ECR, GCP Artifact Registry, and Azure Container Registry. Provider selection
is handled via labels on the composite resource.

Features:
  - Container registry creation with custom configuration
  - Security scanning (vulnerability detection)
  - Image lifecycle policies for automatic cleanup
  - Cross-region replication
  - IAM/access control policies
  - Encryption at rest (KMS support)
  - Pull-through cache (AWS)
  - Network access rules (Azure)
  - Content trust/signing (Azure)

Labels:
  - cloud.provider/aws: "true"    -> Creates AWS ECR repository
  - cloud.provider/gcp: "true"    -> Creates GCP Artifact Registry repository
  - cloud.provider/azure: "true"  -> Creates Azure Container Registry
"""

# Import typed schemas from generated Upbound models
# AWS ECR resources
import models.io.upbound.awsm.ecr.v1beta1 as ecr
import models.io.upbound.awsm.ecr.v1beta2 as ecrv2

# GCP Artifact Registry resources
import models.io.upbound.gcpm.artifact.v1beta1 as gcpartifact
import models.io.upbound.gcpm.artifact.v1beta2 as gcpartifactv2

# Azure Container Registry resources
import models.io.upbound.azure.containerregistry.v1beta1 as azurecr
import models.io.upbound.azure.containerregistry.v1beta2 as azurecrv2

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

schema RegistryParams:
    """Common parameters for all container registry providers"""
    name: str
    region: str
    repositoryName: str
    imageTagMutability: str = "MUTABLE"
    tags: {str: str} = {}

    # Security configuration
    security: SecurityConfig = SecurityConfig {}

    # Lifecycle configuration
    lifecycle: LifecycleConfig = LifecycleConfig {}

    # Replication configuration
    replication: ReplicationConfig = ReplicationConfig {}

    # Access control configuration
    accessControl: AccessControlConfig = AccessControlConfig {}

    # Provider-specific configs
    aws: AWSConfig = None
    gcp: GCPConfig = None
    azure: AzureConfig = None

    check:
        len(name) >= 1 and len(name) <= 63, "Registry name must be 1-63 characters"
        len(repositoryName) >= 2, "Repository name must be at least 2 characters"
        imageTagMutability in ["MUTABLE", "IMMUTABLE"], "Image tag mutability must be MUTABLE or IMMUTABLE"

schema SecurityConfig:
    """Security configuration for container registries"""
    scanOnPush: bool = True
    encryptionType: str = "AES256"
    kmsKeyId: str = ""
    publicAccess: bool = False

    check:
        encryptionType in ["AES256", "KMS"], "Encryption type must be AES256 or KMS"

schema LifecycleConfig:
    """Lifecycle policy configuration"""
    enabled: bool = True
    maxImageAge: int = 14
    maxImageCount: int = 100
    keepTagPatterns: [str] = ["latest", "v*"]

    check:
        maxImageAge >= 1 and maxImageAge <= 365, "Max image age must be 1-365 days"
        maxImageCount >= 1 and maxImageCount <= 10000, "Max image count must be 1-10000"

schema ReplicationConfig:
    """Cross-region replication configuration"""
    enabled: bool = False
    destinationRegions: [str] = []

schema AccessControlConfig:
    """Access control configuration"""
    pullPrincipals: [str] = []
    pushPrincipals: [str] = []
    adminPrincipals: [str] = []

schema AWSConfig:
    """AWS ECR specific configuration"""
    forceDelete: bool = False
    scanType: str = "BASIC"
    pullThroughCache: PullThroughCacheConfig = PullThroughCacheConfig {}

    check:
        scanType in ["BASIC", "ENHANCED"], "Scan type must be BASIC or ENHANCED"

schema PullThroughCacheConfig:
    """Pull-through cache configuration"""
    enabled: bool = False
    upstreamRegistries: [UpstreamRegistry] = []

schema UpstreamRegistry:
    """Upstream registry for pull-through cache"""
    prefix: str
    upstream: str

    check:
        upstream in ["public.ecr.aws", "quay.io", "registry.k8s.io", "docker.io", "ghcr.io"], \
            "Upstream must be a supported registry"

schema GCPConfig:
    """GCP Artifact Registry specific configuration"""
    project: str = ""
    format: str = "DOCKER"
    mode: str = "STANDARD_REPOSITORY"
    cleanupPolicies: CleanupPoliciesConfig = CleanupPoliciesConfig {}

    check:
        format in ["DOCKER", "MAVEN", "NPM", "PYTHON", "APT", "YUM", "GO", "KFP"], \
            "Format must be a valid Artifact Registry format"
        mode in ["STANDARD_REPOSITORY", "REMOTE_REPOSITORY", "VIRTUAL_REPOSITORY"], \
            "Mode must be a valid repository mode"

schema CleanupPoliciesConfig:
    """GCP cleanup policies configuration"""
    enabled: bool = True
    dryRun: bool = False

schema AzureConfig:
    """Azure Container Registry specific configuration"""
    resourceGroup: str
    sku: str = "Standard"
    adminUserEnabled: bool = False
    zoneRedundancy: bool = False
    networkRules: NetworkRulesConfig = NetworkRulesConfig {}
    retentionPolicy: RetentionPolicyConfig = RetentionPolicyConfig {}
    trustPolicy: TrustPolicyConfig = TrustPolicyConfig {}

    check:
        sku in ["Basic", "Standard", "Premium"], "SKU must be Basic, Standard, or Premium"

schema NetworkRulesConfig:
    """Azure network access rules"""
    defaultAction: str = "Allow"
    ipRules: [str] = []
    virtualNetworkRules: [str] = []

    check:
        defaultAction in ["Allow", "Deny"], "Default action must be Allow or Deny"

schema RetentionPolicyConfig:
    """Azure retention policy for untagged manifests"""
    enabled: bool = True
    days: int = 7

    check:
        days >= 0 and days <= 365, "Retention days must be 0-365"

schema TrustPolicyConfig:
    """Azure content trust policy"""
    enabled: bool = False

# ============================================================================
# Main Composition Function
# ============================================================================

createContainerRegistry = lambda oxr: any, ocds: any, params: RegistryParams -> [any] {
    """
    Creates container registry resources based on provider labels.
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
# AWS ECR Resource Creation
# ============================================================================

createAWSResources = lambda params: RegistryParams -> [any] {
    """Creates AWS ECR repository and related resources"""
    _name = "${params.repositoryName}-aws"
    _region = params.region
    _merged_tags = {k: v for k, v in params.tags} | {"ManagedBy": "crossplane", "Provider": "aws"}

    # ECR Repository
    _repository = ecr.Repository {
        metadata = metav1.ObjectMeta {
            name = _name
            labels = {
                "${LABEL_AWS}" = "true"
                "registry.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                region = _region
                imageTagMutability = params.imageTagMutability
                imageScanningConfiguration = {
                    scanOnPush = params.security.scanOnPush
                }
                encryptionConfiguration = {
                    encryptionType = params.security.encryptionType
                    kmsKey = params.security.kmsKeyId if params.security.encryptionType == "KMS" else None
                } if params.security.encryptionType == "KMS" else {
                    encryptionType = "AES256"
                }
                forceDelete = params.aws?.forceDelete or False
                tags = _merged_tags
            }
        }
    }

    # Lifecycle Policy
    _lifecycle_policy = ecr.LifecyclePolicy {
        metadata = metav1.ObjectMeta {
            name = "${_name}-lifecycle"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                repositoryRef = {name = _name}
                policy = _buildAWSLifecyclePolicy(params.lifecycle)
            }
        }
    } if params.lifecycle.enabled else None

    # Repository Policy (IAM-based access control)
    _repo_policy = ecr.RepositoryPolicy {
        metadata = metav1.ObjectMeta {
            name = "${_name}-policy"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                repositoryRef = {name = _name}
                policy = _buildAWSRepositoryPolicy(params.accessControl)
            }
        }
    } if len(params.accessControl.pullPrincipals) > 0 or len(params.accessControl.pushPrincipals) > 0 else None

    # Replication Configuration (account-level)
    _replication = ecr.ReplicationConfiguration {
        metadata = metav1.ObjectMeta {
            name = "${_name}-replication"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                replicationConfiguration = {
                    rule = [
                        {
                            destination = [
                                {region = dest_region}
                                for dest_region in params.replication.destinationRegions
                            ]
                            repositoryFilter = [
                                {
                                    filter = params.repositoryName
                                    filterType = "PREFIX_MATCH"
                                }
                            ]
                        }
                    ]
                }
            }
        }
    } if params.replication.enabled and len(params.replication.destinationRegions) > 0 else None

    # Registry Scanning Configuration
    _scanning_config = ecr.RegistryScanningConfiguration {
        metadata = metav1.ObjectMeta {
            name = "${_name}-scanning"
            labels = {"${LABEL_AWS}": "true"}
        }
        spec = {
            forProvider = {
                region = _region
                scanType = params.aws?.scanType or "BASIC"
                rule = [
                    {
                        scanFrequency = "SCAN_ON_PUSH"
                        repositoryFilter = [
                            {
                                filter = "*"
                                filterType = "WILDCARD"
                            }
                        ]
                    }
                ]
            }
        }
    } if params.security.scanOnPush and params.aws?.scanType == "ENHANCED" else None

    # Pull-Through Cache Rules
    _cache_rules = [
        ecr.PullThroughCacheRule {
            metadata = metav1.ObjectMeta {
                name = "${_name}-cache-${rule.prefix}"
                labels = {"${LABEL_AWS}": "true"}
            }
            spec = {
                forProvider = {
                    region = _region
                    ecrRepositoryPrefix = rule.prefix
                    upstreamRegistryUrl = rule.upstream
                }
            }
        }
        for rule in (params.aws?.pullThroughCache?.upstreamRegistries or [])
    ] if params.aws?.pullThroughCache?.enabled else []

    # Combine all resources
    [r for r in [
        _repository,
        _lifecycle_policy,
        _repo_policy,
        _replication,
        _scanning_config,
    ] + _cache_rules if r]
}

# Helper: Build AWS Lifecycle Policy JSON
_buildAWSLifecyclePolicy = lambda config: LifecycleConfig -> str {
    """Builds AWS ECR lifecycle policy JSON"""
    _keep_patterns = config.keepTagPatterns or ["latest"]
    _rules = [
        {
            rulePriority = 1
            description = "Keep tagged images matching patterns"
            selection = {
                tagStatus = "tagged"
                tagPatternList = _keep_patterns
                countType = "imageCountMoreThan"
                countNumber = config.maxImageCount
            }
            action = {type = "expire"}
        }
        {
            rulePriority = 2
            description = "Expire untagged images older than ${config.maxImageAge} days"
            selection = {
                tagStatus = "untagged"
                countType = "sinceImagePushed"
                countUnit = "days"
                countNumber = config.maxImageAge
            }
            action = {type = "expire"}
        }
    ]
    "{\"rules\": ${_rules}}"
}

# Helper: Build AWS Repository Policy JSON
_buildAWSRepositoryPolicy = lambda config: AccessControlConfig -> str {
    """Builds AWS ECR repository policy JSON"""
    _statements = []

    # Pull access
    _pull_statement = {
        Sid = "AllowPull"
        Effect = "Allow"
        Principal = {AWS = config.pullPrincipals}
        Action = [
            "ecr:GetDownloadUrlForLayer"
            "ecr:BatchGetImage"
            "ecr:BatchCheckLayerAvailability"
        ]
    } if len(config.pullPrincipals) > 0 else None

    # Push access
    _push_statement = {
        Sid = "AllowPush"
        Effect = "Allow"
        Principal = {AWS = config.pushPrincipals}
        Action = [
            "ecr:GetDownloadUrlForLayer"
            "ecr:BatchGetImage"
            "ecr:BatchCheckLayerAvailability"
            "ecr:PutImage"
            "ecr:InitiateLayerUpload"
            "ecr:UploadLayerPart"
            "ecr:CompleteLayerUpload"
        ]
    } if len(config.pushPrincipals) > 0 else None

    _all_statements = [s for s in [_pull_statement, _push_statement] if s]

    "{\"Version\": \"2012-10-17\", \"Statement\": ${_all_statements}}"
}

# ============================================================================
# GCP Artifact Registry Resource Creation
# ============================================================================

createGCPResources = lambda params: RegistryParams -> [any] {
    """Creates GCP Artifact Registry repository and related resources"""
    _name = "${params.repositoryName}-gcp"
    _project = params.gcp?.project if params.gcp else ""
    _format = params.gcp?.format if params.gcp else "DOCKER"
    _mode = params.gcp?.mode if params.gcp else "STANDARD_REPOSITORY"
    _merged_labels = {k.lower().replace("_", "-"): v.lower() for k, v in params.tags} | {"managed-by": "crossplane", "provider": "gcp"}

    # Artifact Registry Repository
    _repository = gcpartifact.RegistryRepository {
        metadata = metav1.ObjectMeta {
            name = _name
            labels = {
                "${LABEL_GCP}" = "true"
                "registry.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                project = _project if _project else None
                location = params.region
                format = _format
                mode = _mode
                description = "Container registry for ${params.name}"
                labels = _merged_labels
                # Cleanup policies
                cleanupPolicies = {
                    "delete-untagged" = {
                        action = "DELETE"
                        condition = {
                            tagState = "UNTAGGED"
                            olderThan = "${params.lifecycle.maxImageAge * 24 * 3600}s"
                        }
                    }
                    "keep-minimum" = {
                        action = "KEEP"
                        mostRecentVersions = {
                            keepCount = params.lifecycle.maxImageCount
                        }
                    }
                } if params.gcp?.cleanupPolicies?.enabled else None
                cleanupPolicyDryRun = params.gcp?.cleanupPolicies?.dryRun if params.gcp?.cleanupPolicies else False
            }
        }
    }

    # IAM Member for Pull Access
    _pull_iam_members = [
        gcpartifact.RegistryRepositoryIamMember {
            metadata = metav1.ObjectMeta {
                name = "${_name}-pull-${i}"
                labels = {"${LABEL_GCP}": "true"}
            }
            spec = {
                forProvider = {
                    project = _project if _project else None
                    location = params.region
                    repositoryRef = {name = _name}
                    role = "roles/artifactregistry.reader"
                    member = principal
                }
            }
        }
        for i, principal in enumerate(params.accessControl.pullPrincipals)
    ]

    # IAM Member for Push Access
    _push_iam_members = [
        gcpartifact.RegistryRepositoryIamMember {
            metadata = metav1.ObjectMeta {
                name = "${_name}-push-${i}"
                labels = {"${LABEL_GCP}": "true"}
            }
            spec = {
                forProvider = {
                    project = _project if _project else None
                    location = params.region
                    repositoryRef = {name = _name}
                    role = "roles/artifactregistry.writer"
                    member = principal
                }
            }
        }
        for i, principal in enumerate(params.accessControl.pushPrincipals)
    ]

    # IAM Member for Admin Access
    _admin_iam_members = [
        gcpartifact.RegistryRepositoryIamMember {
            metadata = metav1.ObjectMeta {
                name = "${_name}-admin-${i}"
                labels = {"${LABEL_GCP}": "true"}
            }
            spec = {
                forProvider = {
                    project = _project if _project else None
                    location = params.region
                    repositoryRef = {name = _name}
                    role = "roles/artifactregistry.admin"
                    member = principal
                }
            }
        }
        for i, principal in enumerate(params.accessControl.adminPrincipals)
    ]

    # Combine all resources
    [r for r in [_repository] + _pull_iam_members + _push_iam_members + _admin_iam_members if r]
}

# ============================================================================
# Azure Container Registry Resource Creation
# ============================================================================

createAzureResources = lambda params: RegistryParams -> [any] {
    """Creates Azure Container Registry and related resources"""
    # ACR names must be alphanumeric only, 5-50 chars
    _name = params.repositoryName.replace("-", "").replace("_", "")[:50]
    _rg = params.azure?.resourceGroup if params.azure else "default-rg"
    _sku = params.azure?.sku if params.azure else "Standard"
    _merged_tags = {k: v for k, v in params.tags} | {"ManagedBy": "crossplane", "Provider": "azure"}

    # Container Registry
    _registry = azurecrv2.Registry {
        metadata = metav1.ObjectMeta {
            name = _name
            labels = {
                "${LABEL_AZURE}" = "true"
                "registry.crossplane.io/name" = params.name
            }
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                location = params.region
                sku = _sku
                adminEnabled = params.azure?.adminUserEnabled or False
                # Zone redundancy (Premium only)
                zoneRedundancyEnabled = params.azure?.zoneRedundancy if _sku == "Premium" else False
                # Public network access
                publicNetworkAccessEnabled = not params.azure?.networkRules?.defaultAction == "Deny" if params.azure?.networkRules else True
                # Network rules (Premium only)
                networkRuleSet = {
                    defaultAction = params.azure?.networkRules?.defaultAction or "Allow"
                    ipRule = [
                        {
                            action = "Allow"
                            ipRange = ip
                        }
                        for ip in (params.azure?.networkRules?.ipRules or [])
                    ]
                    virtualNetworkRule = [
                        {
                            action = "Allow"
                            subnetId = subnet
                        }
                        for subnet in (params.azure?.networkRules?.virtualNetworkRules or [])
                    ]
                } if _sku == "Premium" and params.azure?.networkRules else None
                # Retention policy
                retentionPolicy = {
                    enabled = params.azure?.retentionPolicy?.enabled if params.azure?.retentionPolicy else True
                    days = params.azure?.retentionPolicy?.days if params.azure?.retentionPolicy else 7
                } if _sku in ["Standard", "Premium"] else None
                # Trust policy (content trust)
                trustPolicy = {
                    enabled = params.azure?.trustPolicy?.enabled if params.azure?.trustPolicy else False
                } if _sku == "Premium" else None
                # Quarantine policy for scanning
                quarantinePolicy = {
                    enabled = params.security.scanOnPush
                } if _sku == "Premium" else None
                tags = _merged_tags
            }
        }
    }

    # Geo-replication (Premium only)
    _geo_replications = [
        azurecr.Replication {
            metadata = metav1.ObjectMeta {
                name = "${_name}-repl-${region.replace('-', '')}"
                labels = {"${LABEL_AZURE}": "true"}
            }
            spec = {
                forProvider = {
                    resourceGroupName = _rg
                    registryNameRef = {name = _name}
                    location = region
                    zoneRedundancyEnabled = params.azure?.zoneRedundancy or False
                    tags = _merged_tags
                }
            }
        }
        for region in params.replication.destinationRegions
    ] if params.replication.enabled and _sku == "Premium" else []

    # Scope Map for access control
    _scope_map = azurecr.ScopeMap {
        metadata = metav1.ObjectMeta {
            name = "${_name}-scope"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                containerRegistryNameRef = {name = _name}
                actions = [
                    "repositories/${params.repositoryName}/content/read"
                    "repositories/${params.repositoryName}/content/write"
                    "repositories/${params.repositoryName}/metadata/read"
                ]
            }
        }
    } if len(params.accessControl.pullPrincipals) > 0 or len(params.accessControl.pushPrincipals) > 0 else None

    # Token for programmatic access
    _token = azurecr.Token {
        metadata = metav1.ObjectMeta {
            name = "${_name}-token"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                containerRegistryNameRef = {name = _name}
                scopeMapIdRef = {name = "${_name}-scope"}
                enabled = True
            }
        }
    } if _scope_map else None

    # Webhook for scanning notifications (optional)
    _webhook = azurecr.Webhook {
        metadata = metav1.ObjectMeta {
            name = "${_name}-webhook"
            labels = {"${LABEL_AZURE}": "true"}
        }
        spec = {
            forProvider = {
                resourceGroupName = _rg
                registryNameRef = {name = _name}
                location = params.region
                serviceUri = "https://example.com/webhook"  # Placeholder
                status = "enabled"
                scope = ""  # All repositories
                actions = ["push", "quarantine"]
                tags = _merged_tags
            }
        }
    } if params.security.scanOnPush and _sku == "Premium" else None

    # Combine all resources
    [r for r in [
        _registry,
        _scope_map,
        _token,
        _webhook,
    ] + _geo_replications if r]
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

getRegistryUrl = lambda provider: str, name: str, region: str, project: str -> str {
    """Returns the registry URL for a given provider"""
    if provider == "aws":
        # AWS ECR URL format: <account>.dkr.ecr.<region>.amazonaws.com/<repo>
        "<account>.dkr.ecr.${region}.amazonaws.com/${name}"
    elif provider == "gcp":
        # GCP Artifact Registry URL format: <region>-docker.pkg.dev/<project>/<repo>
        "${region}-docker.pkg.dev/${project}/${name}"
    elif provider == "azure":
        # Azure ACR URL format: <name>.azurecr.io
        "${name}.azurecr.io"
    else:
        ""
}
