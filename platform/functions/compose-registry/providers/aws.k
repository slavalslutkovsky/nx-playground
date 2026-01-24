"""
AWS ECR resource generation for registry composition.
"""

import crossplane_provider_upjet_aws.v1beta2.ecr_aws_upbound_io_v1beta2_repository as ecr
import crossplane_provider_upjet_aws.v1beta1.ecr_aws_upbound_io_v1beta1_lifecycle_policy as ecr_lifecycle
import schemas.registry as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.regions as regions

generate = lambda spec: schema.RegistrySpec, ocds: any -> [any] {
    """Generate AWS ECR resources from RegistrySpec."""
    _name = spec.name
    _region = regions.get_aws_region(spec.region) if spec.region else ""

    _label_config = labels.LabelConfig {
        name = _name
        component = "registry"
        provider = "aws"
        environment = spec.environment
    }
    _tags = labels.generate_tags(_label_config, "aws")
    _k8s_labels = labels.generate_labels(_label_config)

    resources: [any] = []

    # AWS ECR Repository
    resources += [ecr.Repository {
        metadata = h.metadata_with_labels("${_name}-ecr", _k8s_labels)
        spec = {
            forProvider = {
                region = _region
                imageTagMutability = "IMMUTABLE" if spec.immutableTags else "MUTABLE"
                imageScanningConfiguration = {
                    scanOnPush = spec.scanOnPush
                }
                encryptionConfiguration = [{
                    encryptionType = "AES256"
                }] if spec.encryption else []
                tags = _tags
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    # AWS ECR Lifecycle Policy for retention
    if spec.retentionDays > 0:
        resources += [ecr_lifecycle.LifecyclePolicy {
            metadata = h.metadata_with_labels("${_name}-ecr-lifecycle", _k8s_labels)
            spec = {
                forProvider = {
                    region = _region
                    repositorySelector = {
                        matchControllerRef = True
                    }
                    policy = '''{
                        "rules": [
                            {
                                "rulePriority": 1,
                                "description": "Expire untagged images older than ''' + str(spec.retentionDays) + ''' days",
                                "selection": {
                                    "tagStatus": "untagged",
                                    "countType": "sinceImagePushed",
                                    "countUnit": "days",
                                    "countNumber": ''' + str(spec.retentionDays) + '''
                                },
                                "action": {
                                    "type": "expire"
                                }
                            }
                        ]
                    }'''
                }
                deletionPolicy = spec.deletionPolicy
            }
        }]

    resources
}

get_status = lambda spec: schema.RegistrySpec, ocds: any -> schema.RegistryStatus {
    """Extract status from observed composed resources."""
    _observed_ecr = ocds.get("${spec.name}-ecr", {})
    _observed_status = _observed_ecr.get("Resource", {}).get("status", {}).get("atProvider", {})

    _registry_url = _observed_status.get("repositoryUrl", "")

    schema.RegistryStatus {
        provider = "aws"
        region = spec.region
        registryUrl = _registry_url
        repositoryName = spec.name
    }
}
