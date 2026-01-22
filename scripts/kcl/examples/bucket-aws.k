"""
Example: AWS S3 Bucket claim
Run: kcl run scripts/kcl/examples/bucket-aws.k -o yaml
"""

# AWS S3 Bucket with versioning and lifecycle rules
bucket_aws = {
    apiVersion: "storage.platform.io/v1alpha1"
    kind: "Bucket"
    metadata: {
        name: "my-app-data"
        namespace: "default"
    }
    spec: {
        provider: "aws"
        region: "us-east-1"
        environment: "dev"
        name: "my-app-data-bucket"
        storageClass: "standard"
        versioning: True
        encryption: True
        publicAccess: False
        tags: {
            team: "platform"
            project: "my-app"
        }
        lifecycle: [
            {
                name: "archive-old-logs"
                prefix: "logs/"
                transitionDays: 30
                transitionStorageClass: "nearline"
            }
            {
                name: "delete-temp"
                prefix: "tmp/"
                expirationDays: 7
            }
        ]
        cors: [
            {
                allowedOrigins: ["https://app.example.com"]
                allowedMethods: ["GET", "PUT", "POST"]
                allowedHeaders: ["*"]
                maxAgeSeconds: 3600
            }
        ]
    }
}

items = [bucket_aws]
