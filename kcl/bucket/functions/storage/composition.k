"""
Crossplane Composition Function Entry Point

This is the main entry point that Crossplane calls when rendering compositions.
It reads the composite resource, extracts parameters, and delegates to the
appropriate provider handlers based on labels.
"""

import .main as storage

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
_is_aws = _labels.get(storage.LABEL_AWS, "false") == "true"
_is_gcp = _labels.get(storage.LABEL_GCP, "false") == "true"
_is_azure = _labels.get(storage.LABEL_AZURE, "false") == "true"

# Build AWS config if needed
_aws_config = storage.AWSConfig {
    region = _params?.aws?.region or _params?.location
} if _is_aws else None

# Build GCP config if needed
_gcp_config = storage.GCPConfig {
    project = _params?.gcp?.project
    storageClass = _params?.gcp?.storageClass or "STANDARD"
} if _is_gcp else None

# Build Azure config if needed
_azure_config = storage.AzureConfig {
    resourceGroup = _params?.azure?.resourceGroup or "crossplane-rg"
    accountTier = _params?.azure?.accountTier or "Standard"
    accountReplicationType = _params?.azure?.accountReplicationType or "LRS"
} if _is_azure else None

# Build storage parameters from composite resource
storageParams = storage.StorageParams {
    name = _name
    location = _params?.location or "US"
    acl = _params?.acl or "private"
    versioning = _params?.version or False
    tags = _params?.tags or {}
    aws = _aws_config
    gcp = _gcp_config
    azure = _azure_config
}

# Create resources based on provider labels
_desired_resources = storage.createStorageBucket(oxr, ocds, storageParams)

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
