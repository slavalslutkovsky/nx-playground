"""
Generate XNetwork CompositeResourceDefinition from schema.
Run: kcl run render/network_xrd.k > apis/xnetworks/definition.yaml
"""
import schemas.network as network
import schemas.xrd as xrd

_config = xrd.XRDConfig {
    name = network.XRD_NAME
    group = network.XRD_GROUP
    kind = network.XRD_KIND
    plural = network.XRD_PLURAL
    claimKind = network.XRD_CLAIM_KIND
    claimPlural = network.XRD_CLAIM_PLURAL
    compositionName = network.XRD_COMPOSITION
    specProperties = network.SPEC_PROPERTIES
    specRequired = network.SPEC_REQUIRED
    statusProperties = network.STATUS_PROPERTIES
}

[xrd.generate_xrd(_config)]
