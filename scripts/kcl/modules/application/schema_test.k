"""
Tests for application module schema.
Run with: kcl test scripts/kcl/modules/application/
"""

import .schema

# Test ApplicationSpec with storage
test_application_spec_storage_only = lambda {
    spec = schema.ApplicationSpec {
        provider = "aws"
        region = "us-east-1"
        name = "my-app"
        storage = {
            enabled = True
            storageClass = "standard"
        }
    }
    assert spec.name == "my-app"
    assert spec.storage.enabled == True
    assert spec.storage.storageClass == "standard"
}

# Test ApplicationSpec with database
test_application_spec_database_only = lambda {
    spec = schema.ApplicationSpec {
        provider = "gcp"
        region = "us-central1"
        name = "my-app"
        database = {
            enabled = True
            engine = "postgres"
            size = "medium"
        }
    }
    assert spec.database.enabled == True
    assert spec.database.engine == "postgres"
    assert spec.database.size == "medium"
}

# Test ApplicationSpec with both storage and database
test_application_spec_full = lambda {
    spec = schema.ApplicationSpec {
        provider = "aws"
        region = "us-east-1"
        environment = "prod"
        name = "prod-app"
        storage = {
            enabled = True
            storageClass = "nearline"
            versioning = True
            encryption = True
        }
        database = {
            enabled = True
            engine = "postgres"
            size = "large"
            storageGB = 100
            multiAZ = True
            backup = {
                enabled = True
                retentionDays = 14
            }
        }
    }
    assert spec.environment == "prod"
    assert spec.storage.versioning == True
    assert spec.database.multiAZ == True
    assert spec.database.backup.retentionDays == 14
}

# Test StorageConfig defaults
test_storage_config_defaults = lambda {
    config = schema.StorageConfig {}
    assert config.enabled == True
    assert config.storageClass == "standard"
    assert config.versioning == False
    assert config.encryption == True
}

# Test DatabaseConfig defaults
test_database_config_defaults = lambda {
    config = schema.DatabaseConfig {}
    assert config.enabled == True
    assert config.engine == "postgres"
    assert config.size == "small"
    assert config.storageGB == 20
    assert config.multiAZ == False
    assert config.backup.enabled == True
}

# Test BackupConfig defaults
test_backup_config_defaults = lambda {
    config = schema.BackupConfig {}
    assert config.enabled == True
    assert config.retentionDays == 7
}
