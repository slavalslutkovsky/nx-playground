"""
Label generation helpers for Crossplane resources.
Provides standard Kubernetes labels and cloud provider tags.
"""

import .common as c

# Standard Kubernetes label keys
LABEL_APP = "app.kubernetes.io/name"
LABEL_INSTANCE = "app.kubernetes.io/instance"
LABEL_VERSION = "app.kubernetes.io/version"
LABEL_COMPONENT = "app.kubernetes.io/component"
LABEL_PART_OF = "app.kubernetes.io/part-of"
LABEL_MANAGED_BY = "app.kubernetes.io/managed-by"

# Crossplane-specific labels
LABEL_PROVIDER = "crossplane.io/provider"
LABEL_ENVIRONMENT = "crossplane.io/environment"
LABEL_CLAIM_NAME = "crossplane.io/claim-name"
LABEL_CLAIM_NAMESPACE = "crossplane.io/claim-namespace"

schema LabelConfig:
    """Configuration for label generation."""
    name: str
    component?: str
    partOf?: str
    version: str = "v1"
    provider?: str
    environment?: c.Environment
    claimName?: str
    claimNamespace?: str
    extra: {str:str} = {}

generate_labels = lambda config: LabelConfig -> {str:str} {
    """Generate standard Kubernetes labels for a resource."""
    base = {
        "${LABEL_APP}": config.name
        "${LABEL_MANAGED_BY}": "crossplane"
        "${LABEL_VERSION}": config.version
    }
    with_component = base | ({"${LABEL_COMPONENT}": config.component} if config.component else {})
    with_partOf = with_component | ({"${LABEL_PART_OF}": config.partOf} if config.partOf else {})
    with_provider = with_partOf | ({"${LABEL_PROVIDER}": config.provider} if config.provider else {})
    with_env = with_provider | ({"${LABEL_ENVIRONMENT}": config.environment} if config.environment else {})
    with_claim = with_env | ({"${LABEL_CLAIM_NAME}": config.claimName} if config.claimName else {})
    with_ns = with_claim | ({"${LABEL_CLAIM_NAMESPACE}": config.claimNamespace} if config.claimNamespace else {})
    with_ns | config.extra
}

generate_tags = lambda config: LabelConfig, provider: str -> {str:str} {
    """Generate cloud provider tags from label config.

    AWS: preserves label format as-is
    GCP: lowercase, converts "/" and "." to "_" (max 63 chars)
    """
    labels = generate_labels(config)
    labels if provider == "aws" else {
        k.lower().replace("/", "_").replace(".", "_"): v.lower()
        for k, v in labels
        if len(k) <= 63 and len(v) <= 63
    }
}
