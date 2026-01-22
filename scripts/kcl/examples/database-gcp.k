"""
Example: GCP Cloud SQL PostgreSQL Database claim
Run: kcl run scripts/kcl/examples/database-gcp.k -o yaml
"""

# GCP Cloud SQL PostgreSQL Instance
database_gcp = {
    apiVersion: "database.platform.io/v1alpha1"
    kind: "Database"
    metadata: {
        name: "my-app-db"
        namespace: "default"
    }
    spec: {
        provider: "gcp"
        region: "us-central1"
        environment: "staging"
        name: "my-app-cloudsql"
        engine: "postgres"
        size: "medium"
        storageGB: 50
        multiAZ: True  # Regional availability
        publiclyAccessible: False
        backup: {
            enabled: True
            retentionDays: 14
        }
        secrets: {
            generatePassword: True
        }
        tags: {
            team: "backend"
            project: "my-app"
            "cost-center": "engineering"
        }
    }
}

items = [database_gcp]
