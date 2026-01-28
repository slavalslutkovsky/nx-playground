"""
Unit tests for common module.
"""

import .common as common

# ============================================================================
# Region Mapping Tests
# ============================================================================

test_region_aws_to_gcp = lambda {
    """Test AWS to GCP region mapping."""
    assert common.map_region("gcp", "us-east-1") == "us-east1"
    assert common.map_region("gcp", "eu-west-1") == "europe-west1"
    assert common.map_region("gcp", "ap-northeast-1") == "asia-northeast1"
}

test_region_gcp_to_aws = lambda {
    """Test GCP to AWS region mapping."""
    assert common.map_region("aws", "us-east1") == "us-east-1"
    assert common.map_region("aws", "europe-west1") == "eu-west-1"
    assert common.map_region("aws", "asia-northeast1") == "ap-northeast-1"
}

test_region_passthrough = lambda {
    """Test region passthrough when already in correct format."""
    # Already in GCP format, targeting GCP
    assert common.map_region("gcp", "europe-west1") == "europe-west1"
    # Already in AWS format, targeting AWS
    assert common.map_region("aws", "us-east-1") == "us-east-1"
    # Unknown region passes through
    assert common.map_region("aws", "unknown-region") == "unknown-region"
}

# ============================================================================
# BaseSpec Tests
# ============================================================================

test_base_spec_defaults = lambda {
    """Test BaseSpec default values."""
    spec = common.BaseSpec {
        provider = "aws"
    }
    assert spec.provider == "aws"
    assert spec.environment == "dev"
    assert spec.deletionPolicy == "Delete"
    assert len(spec.tags) == 0
}

test_base_spec_with_tags = lambda {
    """Test BaseSpec with tags."""
    spec = common.BaseSpec {
        provider = "gcp"
        tags = {
            "key1" = "value1"
            "key2" = "value2"
        }
    }
    assert len(spec.tags) == 2
    assert spec.tags["key1"] == "value1"
}

# ============================================================================
# Schema Helper Tests
# ============================================================================

test_string_schema = lambda {
    """Test string schema generation."""
    result = common._string_schema("Test description")
    assert result["type"] == "string"
    assert result["description"] == "Test description"
}

test_string_schema_with_enum = lambda {
    """Test string schema with enum values."""
    result = common._string_schema("Provider", ["aws", "gcp"])
    assert result["type"] == "string"
    assert result["enum"] == ["aws", "gcp"]
}

test_string_schema_with_default = lambda {
    """Test string schema with default value."""
    result = common._string_schema("Environment", ["dev", "prod"], "dev")
    assert result["default"] == "dev"
}

test_boolean_schema = lambda {
    """Test boolean schema generation."""
    result = common._boolean_schema("Enable feature", True)
    assert result["type"] == "boolean"
    assert result["description"] == "Enable feature"
    assert result["default"] == True
}

test_integer_schema = lambda {
    """Test integer schema generation."""
    result = common._integer_schema("Storage in GB", 20)
    assert result["type"] == "integer"
    assert result["description"] == "Storage in GB"
    assert result["default"] == 20
}

test_object_schema = lambda {
    """Test object schema generation."""
    result = common._object_schema("Custom tags")
    assert result["type"] == "object"
    assert result["description"] == "Custom tags"
    assert result["additionalProperties"]["type"] == "string"
}

# ============================================================================
# Common Property Schema Tests
# ============================================================================

test_provider_schema = lambda {
    """Test provider schema definitions."""
    assert common.PROVIDER_CLOUD_SCHEMA["type"] == "string"
    assert "aws" in common.PROVIDER_CLOUD_SCHEMA["enum"]
    assert "gcp" in common.PROVIDER_CLOUD_SCHEMA["enum"]
}

test_provider_with_k8s_schema = lambda {
    """Test provider with kubernetes schema definition."""
    assert "kubernetes" in common.PROVIDER_WITH_K8S_SCHEMA["enum"]
}

test_environment_schema = lambda {
    """Test environment schema definition."""
    assert common.ENVIRONMENT_SCHEMA["default"] == "dev"
    assert "dev" in common.ENVIRONMENT_SCHEMA["enum"]
    assert "staging" in common.ENVIRONMENT_SCHEMA["enum"]
    assert "prod" in common.ENVIRONMENT_SCHEMA["enum"]
}

test_deletion_policy_schema = lambda {
    """Test deletion policy schema definition."""
    assert common.DELETION_POLICY_SCHEMA["default"] == "Delete"
    assert "Delete" in common.DELETION_POLICY_SCHEMA["enum"]
    assert "Orphan" in common.DELETION_POLICY_SCHEMA["enum"]
}
