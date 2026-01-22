"""
XRD generator for application module.
Run: kcl run scripts/kcl/modules/application/render/xrd.k
"""

[{
    apiVersion: "apiextensions.crossplane.io/v1"
    kind: "CompositeResourceDefinition"
    metadata: {
        name: "xapplications.platform.io"
        labels: {
            "crossplane.io/module": "application"
        }
    }
    spec: {
        group: "platform.io"
        names: {
            kind: "XApplication"
            plural: "xapplications"
        }
        claimNames: {
            kind: "Application"
            plural: "applications"
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
                                provider: {
                                    type: "string"
                                    enum: ["aws", "gcp"]
                                }
                                environment: {
                                    type: "string"
                                    enum: ["dev", "staging", "prod"]
                                    default: "dev"
                                }
                                region: {type: "string"}
                                deletionPolicy: {
                                    type: "string"
                                    enum: ["Delete", "Orphan"]
                                    default: "Delete"
                                }
                                tags: {
                                    type: "object"
                                    additionalProperties: {type: "string"}
                                }
                                name: {
                                    type: "string"
                                    minLength: 1
                                    maxLength: 63
                                }
                                storage: {
                                    type: "object"
                                    properties: {
                                        enabled: {type: "boolean", default: True}
                                        storageClass: {
                                            type: "string"
                                            enum: ["standard", "nearline", "coldline", "archive"]
                                            default: "standard"
                                        }
                                        versioning: {type: "boolean", default: False}
                                        encryption: {type: "boolean", default: True}
                                    }
                                }
                                database: {
                                    type: "object"
                                    properties: {
                                        enabled: {type: "boolean", default: True}
                                        engine: {
                                            type: "string"
                                            enum: ["postgres", "mysql"]
                                            default: "postgres"
                                        }
                                        size: {
                                            type: "string"
                                            enum: ["small", "medium", "large", "xlarge"]
                                            default: "small"
                                        }
                                        storageGB: {
                                            type: "integer"
                                            minimum: 20
                                            default: 20
                                        }
                                        multiAZ: {type: "boolean", default: False}
                                        backup: {
                                            type: "object"
                                            properties: {
                                                enabled: {type: "boolean", default: True}
                                                retentionDays: {type: "integer", default: 7}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        status: {
                            type: "object"
                            properties: {
                                provider: {type: "string"}
                                region: {type: "string"}
                                applicationName: {type: "string"}
                                storageEnabled: {type: "boolean"}
                                databaseEnabled: {type: "boolean"}
                                storageName: {type: "string"}
                                databaseName: {type: "string"}
                            }
                        }
                    }
                }
            }
        }]
    }
}]
