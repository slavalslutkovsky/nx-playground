"""
Region mapping utilities for multi-cloud resources.
Maps between GCP and AWS region naming conventions.
"""

# GCP region mappings (input -> GCP region)
# Accepts both GCP native names and AWS-style names
GCP_REGION_MAP = {
    # GCP native names (passthrough)
    "europe-west1": "europe-west1"
    "europe-west2": "europe-west2"
    "europe-west3": "europe-west3"
    "europe-west4": "europe-west4"
    "europe-north1": "europe-north1"
    "us-east1": "us-east1"
    "us-east4": "us-east4"
    "us-west1": "us-west1"
    "us-west2": "us-west2"
    "us-central1": "us-central1"
    "asia-east1": "asia-east1"
    "asia-southeast1": "asia-southeast1"
    # AWS-style names -> GCP equivalents
    "eu-west-1": "europe-west1"
    "eu-west-2": "europe-west2"
    "eu-west-3": "europe-west3"
    "eu-north-1": "europe-north1"
    "us-east-1": "us-east1"
    "us-east-2": "us-east4"
    "us-west-1": "us-west1"
    "us-west-2": "us-west2"
    "ap-southeast-1": "asia-southeast1"
    "ap-northeast-1": "asia-east1"
}

# AWS region mappings (input -> AWS region)
# Accepts both AWS native names and GCP-style names
AWS_REGION_MAP = {
    # AWS native names (passthrough)
    "eu-west-1": "eu-west-1"
    "eu-west-2": "eu-west-2"
    "eu-west-3": "eu-west-3"
    "eu-north-1": "eu-north-1"
    "us-east-1": "us-east-1"
    "us-east-2": "us-east-2"
    "us-west-1": "us-west-1"
    "us-west-2": "us-west-2"
    "ap-southeast-1": "ap-southeast-1"
    "ap-northeast-1": "ap-northeast-1"
    # GCP-style names -> AWS equivalents
    "europe-west1": "eu-west-1"
    "europe-west2": "eu-west-2"
    "europe-west3": "eu-west-3"
    "europe-north1": "eu-north-1"
    "us-east1": "us-east-1"
    "us-east4": "us-east-2"
    "us-west1": "us-west-1"
    "us-west2": "us-west-2"
    "us-central1": "us-east-2"
    "asia-southeast1": "ap-southeast-1"
    "asia-east1": "ap-northeast-1"
}

# Default regions
DEFAULT_GCP_REGION = "europe-west1"
DEFAULT_AWS_REGION = "eu-west-1"

# Helper functions for region resolution
get_gcp_region = lambda region: str -> str {
    GCP_REGION_MAP[region] if region in GCP_REGION_MAP else region
}

get_aws_region = lambda region: str -> str {
    AWS_REGION_MAP[region] if region in AWS_REGION_MAP else region
}

# Get region for a specific provider
get_region = lambda region: str, provider: str -> str {
    get_aws_region(region) if provider == "aws" else get_gcp_region(region)
}

# Available availability zones per AWS region
AWS_AVAILABILITY_ZONES = {
    "eu-west-1": ["a", "b", "c"]
    "eu-west-2": ["a", "b", "c"]
    "eu-west-3": ["a", "b", "c"]
    "eu-north-1": ["a", "b", "c"]
    "us-east-1": ["a", "b", "c", "d", "e", "f"]
    "us-east-2": ["a", "b", "c"]
    "us-west-1": ["a", "b"]
    "us-west-2": ["a", "b", "c", "d"]
}

# Get availability zones for an AWS region (defaults to ["a", "b"] if not mapped)
get_aws_azs = lambda region: str -> [str] {
    AWS_AVAILABILITY_ZONES[region] if region in AWS_AVAILABILITY_ZONES else ["a", "b"]
}
