"""
Tests for database module schema.
Run with: kcl test scripts/kcl/modules/database/
"""

import .schema

# Test DatabaseSpec defaults
test_database_spec_defaults = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "my-db"
    }
    assert spec.engine == "postgres"
    assert spec.size == "small"
    assert spec.storageGB == 20
    assert spec.multiAZ == False
    assert spec.publiclyAccessible == False
    assert spec.backup.enabled == True
    assert spec.backup.retentionDays == 7
}

# Test DatabaseSpec with all options
test_database_spec_full = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-central1"
        environment = "prod"
        name = "prod-db"
        engine = "mysql"
        engineVersion = "8.0"
        size = "large"
        storageGB = 100
        multiAZ = True
        backup = {
            enabled = True
            retentionDays = 14
        }
    }
    assert spec.provider == "gcp"
    assert spec.engine == "mysql"
    assert spec.size == "large"
    assert spec.storageGB == 100
    assert spec.multiAZ == True
}

# Test engine version defaults
test_engine_version_default_postgres_aws = lambda {
    result = schema.get_engine_version("postgres", "aws", "")
    assert result == "15.4"
}

test_engine_version_default_postgres_gcp = lambda {
    result = schema.get_engine_version("postgres", "gcp", "")
    assert result == "POSTGRES_15"
}

test_engine_version_custom = lambda {
    result = schema.get_engine_version("postgres", "aws", "14.9")
    assert result == "14.9"
}

# Test instance size mapping
test_instance_class_aws_small = lambda {
    result = schema.get_instance_class("aws", "small")
    assert result == "db.t3.micro"
}

test_instance_class_aws_large = lambda {
    result = schema.get_instance_class("aws", "large")
    assert result == "db.t3.medium"
}

test_instance_class_gcp_small = lambda {
    result = schema.get_instance_class("gcp", "small")
    assert result == "db-f1-micro"
}

test_instance_class_gcp_large = lambda {
    result = schema.get_instance_class("gcp", "large")
    assert result == "db-custom-2-4096"
}

# Test BackupConfig
test_backup_config_defaults = lambda {
    config = schema.BackupConfig {}
    assert config.enabled == True
    assert config.retentionDays == 7
    assert config.preferredWindow == "03:00-04:00"
}

test_backup_config_custom = lambda {
    config = schema.BackupConfig {
        retentionDays = 14
        preferredWindow = "04:00-05:00"
    }
    assert config.retentionDays == 14
    assert config.preferredWindow == "04:00-05:00"
}

# Test SecretsConfig
test_secrets_config_defaults = lambda {
    config = schema.SecretsConfig {}
    assert config.generatePassword == True
    assert config.passwordSecretRef == ""
}
