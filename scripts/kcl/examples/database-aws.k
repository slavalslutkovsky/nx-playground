"""
Example: AWS RDS PostgreSQL Database claim
Run: kcl run scripts/kcl/examples/database-aws.k -o yaml
"""

# AWS RDS PostgreSQL Instance
database_aws = {
    apiVersion: "database.platform.io/v1alpha1"
    kind: "Database"
    metadata: {
        name: "my-app-db"
        namespace: "default"
    }
    spec: {
        provider: "aws"
        region: "us-east-1"
        environment: "dev"
        name: "my-app-postgres"
        engine: "postgres"
        engineVersion: "15.4"
        size: "small"
        storageGB: 20
        storageType: "gp3"
        multiAZ: False
        publiclyAccessible: False
        backup: {
            enabled: True
            retentionDays: 7
            preferredWindow: "03:00-04:00"
        }
        secrets: {
            generatePassword: True
        }
        tags: {
            team: "backend"
            project: "my-app"
        }
    }
}

items = [database_aws]
