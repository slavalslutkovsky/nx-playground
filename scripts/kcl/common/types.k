"""
Common types for Crossplane KCL modules.
Provides shared type definitions used across all modules.
"""

# Provider types
type CloudProvider = "aws" | "gcp"
type Environment = "dev" | "staging" | "prod"
type DeletionPolicy = "Delete" | "Orphan"

schema BaseSpec:
    """Base specification for all Crossplane resources."""
    provider: CloudProvider
    environment: Environment = "dev"
    region: str
    deletionPolicy: DeletionPolicy = "Delete"
    tags: {str:str} = {}

    check:
        len(region) > 0, "region must be specified"

schema ResourceMeta:
    """Metadata for generated resources."""
    name: str
    namespace?: str
    labels: {str:str} = {}
    annotations: {str:str} = {}

    check:
        len(name) > 0, "name must be specified"
        len(name) <= 63, "name must be 63 characters or less"

# Region mappings between providers
AWS_TO_GCP_REGION = {
    "us-east-1": "us-east1"
    "us-east-2": "us-east4"
    "us-west-1": "us-west1"
    "us-west-2": "us-west2"
    "eu-west-1": "europe-west1"
    "eu-west-2": "europe-west2"
    "eu-central-1": "europe-west3"
    "ap-northeast-1": "asia-northeast1"
    "ap-southeast-1": "asia-southeast1"
    "ap-south-1": "asia-south1"
}

GCP_TO_AWS_REGION = {v: k for k, v in AWS_TO_GCP_REGION}

map_region = lambda provider: CloudProvider, region: str -> str {
    """Map region to provider-specific format."""
    AWS_TO_GCP_REGION[region] if provider == "gcp" and region in AWS_TO_GCP_REGION else (GCP_TO_AWS_REGION[region] if provider == "aws" and region in GCP_TO_AWS_REGION else region)
}

# Size mappings
SIZE_TO_INSTANCE = {
    "aws": {
        "small": "db.t3.micro"
        "medium": "db.t3.small"
        "large": "db.t3.medium"
        "xlarge": "db.t3.large"
    }
    "gcp": {
        "small": "db-f1-micro"
        "medium": "db-g1-small"
        "large": "db-custom-2-4096"
        "xlarge": "db-custom-4-8192"
    }
}

get_instance_size = lambda provider: CloudProvider, size: str -> str {
    """Get provider-specific instance size."""
    SIZE_TO_INSTANCE[provider][size] if size in SIZE_TO_INSTANCE[provider] else SIZE_TO_INSTANCE[provider]["small"]
}
