"""
Example: Full Application Stack with Storage and Database
Run: kcl run scripts/kcl/examples/application-full.k -o yaml

This creates:
- XApplication (parent composite)
  - XBucket -> S3 Bucket + policies (AWS)
  - XDatabase -> RDS Instance + subnet group (AWS)
"""

# Full Application Stack on AWS
application_aws = {
    apiVersion: "platform.io/v1alpha1"
    kind: "Application"
    metadata: {
        name: "my-app"
        namespace: "default"
    }
    spec: {
        provider: "aws"
        region: "us-east-1"
        environment: "prod"
        name: "my-app"
        tags: {
            team: "platform"
            project: "my-app"
            "cost-center": "engineering"
        }
        storage: {
            enabled: True
            storageClass: "standard"
            versioning: True
            encryption: True
        }
        database: {
            enabled: True
            engine: "postgres"
            size: "medium"
            storageGB: 100
            multiAZ: True
            backup: {
                enabled: True
                retentionDays: 14
            }
        }
    }
}

# Full Application Stack on GCP
application_gcp = {
    apiVersion: "platform.io/v1alpha1"
    kind: "Application"
    metadata: {
        name: "my-app-gcp"
        namespace: "default"
    }
    spec: {
        provider: "gcp"
        region: "us-central1"
        environment: "prod"
        name: "my-app-gcp"
        tags: {
            team: "platform"
            project: "my-app"
        }
        storage: {
            enabled: True
            storageClass: "nearline"
            versioning: False
            encryption: True
        }
        database: {
            enabled: True
            engine: "postgres"
            size: "large"
            storageGB: 200
            multiAZ: True
            backup: {
                enabled: True
                retentionDays: 30
            }
        }
    }
}

# Application with only storage (no database)
application_storage_only = {
    apiVersion: "platform.io/v1alpha1"
    kind: "Application"
    metadata: {
        name: "static-site"
        namespace: "default"
    }
    spec: {
        provider: "aws"
        region: "us-west-2"
        environment: "dev"
        name: "static-site"
        storage: {
            enabled: True
            storageClass: "standard"
            versioning: False
            encryption: True
        }
    }
}

# Application with only database (no storage)
application_db_only = {
    apiVersion: "platform.io/v1alpha1"
    kind: "Application"
    metadata: {
        name: "api-service"
        namespace: "default"
    }
    spec: {
        provider: "gcp"
        region: "europe-west1"
        environment: "staging"
        name: "api-service"
        database: {
            enabled: True
            engine: "mysql"
            size: "small"
            storageGB: 20
            multiAZ: False
        }
    }
}

items = [
    application_aws
    application_gcp
    application_storage_only
    application_db_only
]
