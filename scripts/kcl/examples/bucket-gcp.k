"""
Example: GCP Cloud Storage Bucket claim
Run: kcl run scripts/kcl/examples/bucket-gcp.k -o yaml
"""

# GCP Cloud Storage Bucket
bucket_gcp = {
    apiVersion: "storage.platform.io/v1alpha1"
    kind: "Bucket"
    metadata: {
        name: "my-app-assets"
        namespace: "default"
    }
    spec: {
        provider: "gcp"
        region: "us-central1"
        environment: "staging"
        name: "my-app-assets-bucket"
        storageClass: "nearline"
        versioning: False
        encryption: True
        publicAccess: False
        tags: {
            team: "frontend"
            project: "my-app"
            "cost-center": "marketing"
        }
        lifecycle: [
            {
                name: "archive-to-coldline"
                transitionDays: 90
                transitionStorageClass: "coldline"
            }
        ]
    }
}

items = [bucket_gcp]
