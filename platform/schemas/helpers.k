"""
Shared helper functions for Crossplane composition functions.
"""

# Crossplane function input extraction
# Usage: oxr, ocds, dxr, dcds = get_function_inputs()
get_function_inputs = lambda -> any {
    {
        oxr = option("params").oxr
        ocds = option("params").ocds
        dxr = option("params").dxr
        dcds = option("params").dcds
    }
}

# Helper for resource naming annotation (required for Crossplane composition)
# Creates metadata with the krm.kcl.dev/composition-resource-name annotation
metadata = lambda name: str, r_name: str -> {str:any} {
    {
        name = name
        annotations = {
            "krm.kcl.dev/composition-resource-name" = name
        }
    }
}

# Helper for metadata with additional labels
metadata_with_labels = lambda name: str, labels: {str:str} -> {str:any} {
    {
        name = name
        annotations = {
            "krm.kcl.dev/composition-resource-name" = name
        }
        labels = labels
    }
}

# Generate AWS-style tags (uses dashes)
aws_tags = lambda environment: str, extra: {str:str} = {} -> {str:str} {
    {
        "managed-by" = "crossplane"
        "environment" = environment
    } | extra
}

# Generate GCP-style labels (uses underscores, lowercase)
gcp_labels = lambda environment: str, extra: {str:str} = {} -> {str:str} {
    {
        "managed_by" = "crossplane"
        "environment" = environment
    } | extra
}

# Generate Kubernetes-style labels
k8s_labels = lambda environment: str, extra: {str:str} = {} -> {str:str} {
    {
        "managed-by" = "crossplane"
        "environment" = environment
    } | extra
}

# Build the desired composite resource (dxr) with status update
build_dxr = lambda oxr: any, status: any -> {str:any} {
    {
        **oxr
        status = status
    }
}

# Check if a value is None or empty string
is_empty = lambda value: any -> bool {
    value == None or value == ""
}

# Get value with default (useful for optional spec fields)
get_or_default = lambda value: any, default: any -> any {
    default if is_empty(value) else value
}
