"""
Unit tests for labels module.
"""

import .labels as labels

# ============================================================================
# Label Generation Tests
# ============================================================================

test_generate_labels_basic = lambda {
    """Test basic label generation."""
    config = labels.LabelConfig {
        name = "my-app"
    }
    result = labels.generate_labels(config)

    assert result[labels.LABEL_APP] == "my-app"
    assert result[labels.LABEL_MANAGED_BY] == "crossplane"
    assert result[labels.LABEL_VERSION] == "v1"
}

test_generate_labels_with_component = lambda {
    """Test label generation with component."""
    config = labels.LabelConfig {
        name = "my-app"
        component = "database"
    }
    result = labels.generate_labels(config)

    assert result[labels.LABEL_APP] == "my-app"
    assert result[labels.LABEL_COMPONENT] == "database"
}

test_generate_labels_with_environment = lambda {
    """Test label generation with environment."""
    config = labels.LabelConfig {
        name = "my-app"
        environment = "prod"
    }
    result = labels.generate_labels(config)

    assert result[labels.LABEL_ENVIRONMENT] == "prod"
}

test_generate_labels_with_provider = lambda {
    """Test label generation with provider."""
    config = labels.LabelConfig {
        name = "my-app"
        provider = "aws"
    }
    result = labels.generate_labels(config)

    assert result[labels.LABEL_PROVIDER] == "aws"
}

test_generate_labels_with_extra = lambda {
    """Test label generation with extra labels."""
    config = labels.LabelConfig {
        name = "my-app"
        extra = {
            "custom-label" = "custom-value"
        }
    }
    result = labels.generate_labels(config)

    assert result["custom-label"] == "custom-value"
}

# ============================================================================
# Tag Generation Tests
# ============================================================================

test_generate_tags_aws = lambda {
    """Test AWS tag generation preserves format."""
    config = labels.LabelConfig {
        name = "my-app"
        component = "storage"
    }
    result = labels.generate_tags(config, "aws")

    # AWS preserves the label format
    assert result[labels.LABEL_APP] == "my-app"
    assert result[labels.LABEL_COMPONENT] == "storage"
}

test_generate_tags_gcp = lambda {
    """Test GCP tag generation transforms keys."""
    config = labels.LabelConfig {
        name = "my-app"
        component = "storage"
        provider = "gcp"
    }
    result = labels.generate_tags(config, "gcp")

    # GCP transforms: lowercase, / and . to _
    assert "app_kubernetes_io_name" in result
    assert result["app_kubernetes_io_name"] == "my-app"
}

# ============================================================================
# Label Key Constants Tests
# ============================================================================

test_label_keys = lambda {
    """Test that label key constants are correctly defined."""
    assert labels.LABEL_APP == "app.kubernetes.io/name"
    assert labels.LABEL_INSTANCE == "app.kubernetes.io/instance"
    assert labels.LABEL_VERSION == "app.kubernetes.io/version"
    assert labels.LABEL_COMPONENT == "app.kubernetes.io/component"
    assert labels.LABEL_PART_OF == "app.kubernetes.io/part-of"
    assert labels.LABEL_MANAGED_BY == "app.kubernetes.io/managed-by"
    assert labels.LABEL_PROVIDER == "crossplane.io/provider"
    assert labels.LABEL_ENVIRONMENT == "crossplane.io/environment"
}
