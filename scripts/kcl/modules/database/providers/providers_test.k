"""
Tests for database module providers.
Run with: kcl test scripts/kcl/modules/database/
"""

import ..schema
import .aws
import .gcp

# Test AWS RDS generation - basic
test_aws_generate_basic = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-db"
    }
    resources = aws.generate(spec)

    # Should have subnet group, parameter group, and instance
    assert len(resources) >= 3

    # Check main instance
    instance = [r for r in resources if r.kind == "Instance"][0]
    assert instance.metadata.name == "test-db"
    assert instance.spec.forProvider.engine == "postgres"
    assert instance.spec.forProvider.instanceClass == "db.t3.micro"
}

# Test AWS RDS generation - with custom size
test_aws_generate_large = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-db"
        size = "large"
    }
    resources = aws.generate(spec)

    instance = [r for r in resources if r.kind == "Instance"][0]
    assert instance.spec.forProvider.instanceClass == "db.t3.medium"
}

# Test AWS RDS generation - MySQL
test_aws_generate_mysql = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-db"
        engine = "mysql"
    }
    resources = aws.generate(spec)

    instance = [r for r in resources if r.kind == "Instance"][0]
    assert instance.spec.forProvider.engine == "mysql"
}

# Test AWS RDS generation - Multi-AZ
test_aws_generate_multiaz = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-db"
        multiAZ = True
    }
    resources = aws.generate(spec)

    instance = [r for r in resources if r.kind == "Instance"][0]
    assert instance.spec.forProvider.multiAz == True
}

# Test AWS RDS generation - backup configuration
test_aws_generate_backup = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-db"
        backup = {
            enabled = True
            retentionDays = 14
            preferredWindow = "04:00-05:00"
        }
    }
    resources = aws.generate(spec)

    instance = [r for r in resources if r.kind == "Instance"][0]
    assert instance.spec.forProvider.backupRetentionPeriod == 14
    assert instance.spec.forProvider.backupWindow == "04:00-05:00"
}

# Test AWS RDS generation - prod environment (no skip snapshot)
test_aws_generate_prod = lambda {
    spec = schema.DatabaseSpec {
        provider = "aws"
        region = "us-east-1"
        name = "test-db"
        environment = "prod"
    }
    resources = aws.generate(spec)

    instance = [r for r in resources if r.kind == "Instance"][0]
    assert instance.spec.forProvider.skipFinalSnapshot == False
}

# Test GCP Cloud SQL generation - basic
test_gcp_generate_basic = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-east-1"
        name = "test-db"
    }
    resources = gcp.generate(spec)

    # Should have instance, database, and user
    assert len(resources) >= 3

    # Check main instance
    instance = [r for r in resources if r.kind == "DatabaseInstance"][0]
    assert instance.metadata.name == "test-db"
    assert instance.spec.forProvider.databaseVersion == "POSTGRES_15"
}

# Test GCP Cloud SQL generation - MySQL
test_gcp_generate_mysql = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-db"
        engine = "mysql"
    }
    resources = gcp.generate(spec)

    instance = [r for r in resources if r.kind == "DatabaseInstance"][0]
    assert instance.spec.forProvider.databaseVersion == "MYSQL_8_0"
}

# Test GCP Cloud SQL generation - Multi-AZ (REGIONAL)
test_gcp_generate_multiaz = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-db"
        multiAZ = True
    }
    resources = gcp.generate(spec)

    instance = [r for r in resources if r.kind == "DatabaseInstance"][0]
    assert instance.spec.forProvider.settings[0].availabilityType == "REGIONAL"
}

# Test GCP Cloud SQL generation - storage settings
test_gcp_generate_storage = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-db"
        storageGB = 100
    }
    resources = gcp.generate(spec)

    instance = [r for r in resources if r.kind == "DatabaseInstance"][0]
    assert instance.spec.forProvider.settings[0].diskSize == 100
    assert instance.spec.forProvider.settings[0].diskAutoresizeLimit == 200
}

# Test GCP Cloud SQL generation - creates database and user
test_gcp_generate_creates_db_user = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-central1"
        name = "test-db"
    }
    resources = gcp.generate(spec)

    databases = [r for r in resources if r.kind == "Database"]
    users = [r for r in resources if r.kind == "User"]

    assert len(databases) == 1
    assert len(users) == 1
}

# Test GCP Cloud SQL - region mapping
test_gcp_region_mapping = lambda {
    spec = schema.DatabaseSpec {
        provider = "gcp"
        region = "us-east-1"
        name = "test-db"
    }
    resources = gcp.generate(spec)

    instance = [r for r in resources if r.kind == "DatabaseInstance"][0]
    # us-east-1 should be mapped to us-east1
    assert instance.spec.forProvider.region == "us-east1"
}
