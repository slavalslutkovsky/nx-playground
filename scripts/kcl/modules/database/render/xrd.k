"""
XRD generator for database module.
Run: kcl run scripts/kcl/modules/database/render/xrd.k
"""

[{
    apiVersion: "apiextensions.crossplane.io/v1"
    kind: "CompositeResourceDefinition"
    metadata: {
        name: "xdatabases.database.platform.io"
        labels: {
            "crossplane.io/module": "database"
        }
    }
    spec: {
        group: "database.platform.io"
        names: {
            kind: "XDatabase"
            plural: "xdatabases"
        }
        claimNames: {
            kind: "Database"
            plural: "databases"
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
                                    description: "Cloud provider (aws or gcp)"
                                }
                                environment: {
                                    type: "string"
                                    enum: ["dev", "staging", "prod"]
                                    default: "dev"
                                }
                                region: {
                                    type: "string"
                                }
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
                                engine: {
                                    type: "string"
                                    enum: ["postgres", "mysql"]
                                    default: "postgres"
                                }
                                engineVersion: {
                                    type: "string"
                                }
                                size: {
                                    type: "string"
                                    enum: ["small", "medium", "large", "xlarge"]
                                    default: "small"
                                }
                                storageGB: {
                                    type: "integer"
                                    minimum: 20
                                    maximum: 65536
                                    default: 20
                                }
                                multiAZ: {
                                    type: "boolean"
                                    default: False
                                }
                                publiclyAccessible: {
                                    type: "boolean"
                                    default: False
                                }
                                backup: {
                                    type: "object"
                                    properties: {
                                        enabled: {
                                            type: "boolean"
                                            default: True
                                        }
                                        retentionDays: {
                                            type: "integer"
                                            minimum: 0
                                            maximum: 35
                                            default: 7
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
                                engine: {type: "string"}
                                instanceName: {type: "string"}
                                size: {type: "string"}
                            }
                        }
                    }
                }
            }
        }]
    }
}]
