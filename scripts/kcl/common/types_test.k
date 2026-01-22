"""
Tests for common types module.
Run with: kcl test scripts/kcl/common/
"""

import .types
import .labels

# Test BaseSpec validation
test_base_spec_defaults = lambda {
    spec = types.BaseSpec {
        provider = "aws"
        region = "us-east-1"
    }
    assert spec.environment == "dev"
    assert spec.deletionPolicy == "Delete"
    assert spec.tags == {}
}

test_base_spec_full = lambda {
    spec = types.BaseSpec {
        provider = "gcp"
        environment = "prod"
        region = "us-central1"
        deletionPolicy = "Orphan"
        tags = {
            team: "platform"
            "cost-center": "engineering"
        }
    }
    assert spec.provider == "gcp"
    assert spec.environment == "prod"
    assert spec.deletionPolicy == "Orphan"
    assert spec.tags["team"] == "platform"
}

# Test region mapping
test_region_mapping_aws_to_gcp = lambda {
    result = types.map_region("gcp", "us-east-1")
    assert result == "us-east1"
}

test_region_mapping_gcp_to_aws = lambda {
    result = types.map_region("aws", "us-east1")
    assert result == "us-east-1"
}

test_region_mapping_unknown = lambda {
    result = types.map_region("aws", "unknown-region")
    assert result == "unknown-region"
}

# Test instance size mapping
test_instance_size_aws = lambda {
    result = types.get_instance_size("aws", "small")
    assert result == "db.t3.micro"
}

test_instance_size_gcp = lambda {
    result = types.get_instance_size("gcp", "medium")
    assert result == "db-g1-small"
}

# Test label generation
test_generate_labels_basic = lambda {
    config = labels.LabelConfig {
        name = "my-app"
    }
    result = labels.generate_labels(config)
    assert result[labels.LABEL_APP] == "my-app"
    assert result[labels.LABEL_MANAGED_BY] == "crossplane"
}

test_generate_labels_full = lambda {
    config = labels.LabelConfig {
        name = "my-app"
        component = "storage"
        partOf = "my-system"
        provider = "aws"
        environment = "prod"
        extra = {
            "custom-label": "custom-value"
        }
    }
    result = labels.generate_labels(config)
    assert result[labels.LABEL_APP] == "my-app"
    assert result[labels.LABEL_COMPONENT] == "storage"
    assert result[labels.LABEL_PART_OF] == "my-system"
    assert result[labels.LABEL_PROVIDER] == "aws"
    assert result[labels.LABEL_ENVIRONMENT] == "prod"
    assert result["custom-label"] == "custom-value"
}

# Test tag generation for different providers
test_generate_tags_aws = lambda {
    config = labels.LabelConfig {
        name = "my-bucket"
        provider = "aws"
    }
    result = labels.generate_tags(config, "aws")
    assert result[labels.LABEL_APP] == "my-bucket"
}

test_generate_tags_gcp = lambda {
    config = labels.LabelConfig {
        name = "my-bucket"
        provider = "gcp"
    }
    result = labels.generate_tags(config, "gcp")
    # GCP converts to lowercase and replaces special chars
    assert "app_kubernetes_io_name" in result
}

# Test ResourceMeta
test_resource_meta = lambda {
    meta = types.ResourceMeta {
        name = "my-resource"
        namespace = "default"
        labels = {
            "app": "test"
        }
    }
    assert meta.name == "my-resource"
    assert meta.namespace == "default"
    assert meta.labels["app"] == "test"
}
