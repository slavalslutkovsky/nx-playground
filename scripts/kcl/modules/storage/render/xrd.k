"""
XRD generator for storage module.
Generates the CompositeResourceDefinition YAML.
Run: kcl run scripts/kcl/modules/storage/render/xrd.k
"""

# XRD for XBucket - output directly (no variable assignment for clean output)
[{
    apiVersion: "apiextensions.crossplane.io/v1"
    kind: "CompositeResourceDefinition"
    metadata: {
        name: "xbuckets.storage.platform.io"
        labels: {
            "crossplane.io/module": "storage"
        }
    }
    spec: {
        group: "storage.platform.io"
        names: {
            kind: "XBucket"
            plural: "xbuckets"
        }
        claimNames: {
            kind: "Bucket"
            plural: "buckets"
        }
        versions: [{
            name: "v1alpha1"
            served: True
            referenceable: True
            schema: {
                openAPIV3Schema: {
                    type: "object"
                    properties: {
                        spec: {
                            type: "object"
                            required: ["provider", "region", "name"]
                            properties: {
                                id: {
                                  type: "string"
                                  description: "Resource id which will be the name in the cloud"
                                }
                                provider: {
                                    type: "string"
                                    enum: ["aws", "gcp"]
                                    description: "Cloud provider (aws or gcp)"
                                }
                                paramas: {
                                  type: "object"
                                  description: "Params of the cloud resource shared to all providers"
                                  required: ["provider", "region", "name"]
                                  properties: {
                                       provider: {
                                          type: "string"
                                          enum: ["aws", "gcp"]
                                          description: "Cloud provider (aws or gcp)"
                                       }
                                       region: {
                                           type: "string"
                                           description: "Cloud region for the bucket"
                                       }
                                       name: {
                                           type: "string"
                                           minLength: 3
                                           maxLength: 63
                                           description: "Name of the bucket"
                                       }
                                  }
                                }
                                environment: {
                                    type: "string"
                                    enum: ["dev", "staging", "prod"]
                                    default: "dev"
                                    description: "Deployment environment"
                                }
                                region: {
                                    type: "string"
                                    description: "Cloud region for the bucket"
                                }
                                deletionPolicy: {
                                    type: "string"
                                    enum: ["Delete", "Orphan"]
                                    default: "Delete"
                                    description: "Deletion policy for the bucket"
                                }
                                tags: {
                                    type: "object"
                                    additionalProperties: {type: "string"}
                                    description: "Tags/labels for the bucket"
                                }
                                name: {
                                    type: "string"
                                    minLength: 3
                                    maxLength: 63
                                    description: "Name of the bucket"
                                }
                                storageClass: {
                                    type: "string"
                                    enum: ["standard", "nearline", "coldline", "archive"]
                                    default: "standard"
                                    description: "Storage class for the bucket"
                                }
                                versioning: {
                                    type: "boolean"
                                    default: False
                                    description: "Enable versioning for the bucket"
                                }
                                encryption: {
                                    type: "boolean"
                                    default: True
                                    description: "Enable server-side encryption"
                                }
                                publicAccess: {
                                    type: "boolean"
                                    default: False
                                    description: "Allow public access to the bucket"
                                }
                            }
                        }
                        status: {
                            type: "object"
                            properties: {
                                provider: {
                                    type: "string"
                                    description: "Cloud provider used"
                                }
                                region: {
                                    type: "string"
                                    description: "Region where bucket is deployed"
                                }
                                bucketName: {
                                    type: "string"
                                    description: "Name of the created bucket"
                                }
                            }
                        }
                    }
                }
            }
        }]
    }
}]
