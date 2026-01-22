"""
Generate XRegistry CompositeResourceDefinition from schema.
Run: kcl run render/registry_xrd.k > apis/xregistries/definition.yaml
"""
import schemas.registry as registry
import schemas.xrd as xrd

_config = xrd.XRDConfig {
    name = registry.XRD_NAME
    group = registry.XRD_GROUP
    kind = registry.XRD_KIND
    plural = registry.XRD_PLURAL
    claimKind = registry.XRD_CLAIM_KIND
    claimPlural = registry.XRD_CLAIM_PLURAL
    compositionName = registry.XRD_COMPOSITION
    specProperties = registry.SPEC_PROPERTIES
    specRequired = registry.SPEC_REQUIRED
    statusProperties = registry.STATUS_PROPERTIES
}

[xrd.generate_xrd(_config)]
