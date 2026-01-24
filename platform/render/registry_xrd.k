"""
Generate XRegistry CompositeResourceDefinition from schema.
Run: kcl run render/registry_xrd.k > apis/xregistries/definition.yaml
"""
import schemas.registry as registry
import schemas.xrd as xrd

_config = xrd.XRDConfig {
    name = registry.REGISTRY_XRD_NAME
    group = registry.REGISTRY_XRD_GROUP
    kind = registry.REGISTRY_XRD_KIND
    plural = registry.REGISTRY_XRD_PLURAL
    claimKind = registry.REGISTRY_XRD_CLAIM_KIND
    claimPlural = registry.REGISTRY_XRD_CLAIM_PLURAL
    compositionName = registry.REGISTRY_XRD_COMPOSITION
    specProperties = registry.REGISTRY_SPEC_PROPERTIES
    specRequired = registry.REGISTRY_SPEC_REQUIRED
    statusProperties = registry.REGISTRY_STATUS_PROPERTIES
}

[xrd.generate_xrd(_config)]
