"""
Unit tests for database db.
"""

import .database as db

# ============================================================================
# Schema Validation Tests
# ============================================================================

test_database_spec_basic_aws = lambda {
    """Test basic AWS database spec creation."""
    spec = db.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        engine = "postgres"
    }
    assert spec.provider == "aws"
    assert spec.region == "us-east-1"
    assert spec.engine == "postgres"
    assert spec.engineVersion == "18"
    assert spec.size == "small"
    assert spec.storageGB == 20
    assert spec.multiAZ == False
    assert spec.publiclyAccessible == False
}

test_database_spec_basic_gcp = lambda {
    """Test basic GCP database spec creation."""
    spec = db.DatabaseSpec {
        provider = "gcp"
        region = "europe-west1"
        engine = "mysql"
        size = "medium"
    }
    assert spec.provider == "gcp"
    assert spec.engine == "mysql"
    assert spec.size == "medium"
}

test_database_spec_kubernetes = lambda {
    """Test Kubernetes/CNPG database spec creation."""
    spec = db.DatabaseSpec {
        provider = "kubernetes"
        engine = "postgres"
        instances = 3
        namespace = "production"
    }
    assert spec.provider == "kubernetes"
    assert spec.engine == "postgres"
    assert spec.instances == 3
    assert spec.namespace == "production"
}

test_database_spec_with_backup = lambda {
    """Test database spec with backup configuration."""
    spec = db.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        backup = db.BackupConfig {
            enabled = True
            retentionDays = 14
            preferredWindow = "04:00-05:00"
        }
    }
    assert spec.backup.enabled == True
    assert spec.backup.retentionDays == 14
    assert spec.backup.preferredWindow == "04:00-05:00"
}

test_database_spec_with_secrets = lambda {
    """Test database spec with secrets configuration."""
    spec = db.DatabaseSpec {
        provider = "gcp"
        region = "us-east1"
        secrets = db.SecretsConfig {
            generatePassword = False
            passwordSecretRef = "my-db-password"
        }
    }
    assert spec.secrets.generatePassword == False
    assert spec.secrets.passwordSecretRef == "my-db-password"
}

# ============================================================================
# Instance Class Mapping Tests
# ============================================================================

test_instance_class_aws = lambda {
    """Test AWS instance class mapping."""
    assert db.get_instance_class("aws", "small") == "db.t3.micro"
    assert db.get_instance_class("aws", "medium") == "db.t3.small"
    assert db.get_instance_class("aws", "large") == "db.t3.medium"
    assert db.get_instance_class("aws", "xlarge") == "db.t3.large"
}

test_instance_class_gcp = lambda {
    """Test GCP instance class mapping."""
    assert db.get_instance_class("gcp", "small") == "db-f1-micro"
    assert db.get_instance_class("gcp", "medium") == "db-g1-small"
    assert db.get_instance_class("gcp", "large") == "db-custom-2-4096"
    assert db.get_instance_class("gcp", "xlarge") == "db-custom-4-8192"
}

test_instance_class_kubernetes = lambda {
    """Test Kubernetes resource mapping."""
    resources = db.get_instance_class("kubernetes", "small")
    assert resources["cpu"] == "100m"
    assert resources["memory"] == "256Mi"
}

# ============================================================================
# Engine Version Tests
# ============================================================================

test_engine_version_defaults = lambda {
    """Test engine version defaults."""
    assert db.get_engine_version("postgres", "aws", "") == "15.4"
    assert db.get_engine_version("mysql", "aws", "") == "8.0.35"
}

test_engine_version_custom = lambda {
    """Test custom engine version."""
    assert db.get_engine_version("postgres", "aws", "14.5") == "14.5"
    assert db.get_engine_version("mysql", "gcp", "8.0") == "8.0"
}

# ============================================================================
# Storage Type Tests
# ============================================================================

test_storage_type_aws = lambda {
    """Test AWS storage type mapping."""
    assert db.get_storage_type("aws", "gp3") == "gp3"
    assert db.get_storage_type("aws", "gp2") == "gp2"
    assert db.get_storage_type("aws", "io1") == "io1"
}

test_storage_type_gcp = lambda {
    """Test GCP storage type mapping."""
    assert db.get_storage_type("gcp", "ssd") == "SSD"
    assert db.get_storage_type("gcp", "standard") == "SSD"
}

# ============================================================================
# Tags Field Tests
# ============================================================================

test_database_spec_with_tags = lambda {
    """Test database spec with custom tags."""
    spec = db.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        tags = {
            "cost-center" = "engineering"
            "owner" = "platform-team"
        }
    }
    assert len(spec.tags) == 2
    assert spec.tags["cost-center"] == "engineering"
}
