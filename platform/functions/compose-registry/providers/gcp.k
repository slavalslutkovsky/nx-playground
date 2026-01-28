"""
GCP Artifact Registry resource generation for registry composition.
"""

import crossplane_provider_upjet_gcp.v1beta1.artifact_gcp_upbound_io_v1beta1_registry_repository as artifact
import schemas.registry as schema
import schemas.labels as labels
import schemas.helpers as h
import schemas.regions as regions

generate = lambda spec: schema.RegistrySpec, ocds: any -> [any] {
    """Generate GCP Artifact Registry resources from RegistrySpec."""
    _name = spec.name
    _region = regions.get_gcp_region(spec.region) if spec.region else ""

    _label_config = labels.LabelConfig {
        name = _name
        component = "registry"
        provider = "gcp"
        environment = spec.environment
    }
    _gcp_labels = labels.generate_tags(_label_config, "gcp")
    _k8s_labels = labels.generate_labels(_label_config)

    resources: [any] = []

    # GCP Artifact Registry Repository
    resources += [artifact.RegistryRepository {
        metadata = h.metadata_with_labels("${_name}-artifactregistry", _k8s_labels)
        spec = {
            forProvider = {
                location = _region
                format = spec.format.upper()
                mode = "STANDARD_REPOSITORY"
                labels = _gcp_labels
            }
            deletionPolicy = spec.deletionPolicy
        }
    }]

    resources
}

get_status = lambda spec: schema.RegistrySpec, ocds: any -> schema.RegistryStatus {
    """Extract status from observed composed resources."""
    _observed_artifact = ocds.get("${spec.name}-artifactregistry", {})
    _observed_status = _observed_artifact.get("Resource", {}).get("status", {}).get("atProvider", {})

    _name = _observed_status.get("name", "")
    _region = spec.region

    schema.RegistryStatus {
        provider = "gcp"
        region = _region
        registryUrl = "${_region}-docker.pkg.dev/${_name}" if _name else ""
        repositoryName = spec.name
    }
}
