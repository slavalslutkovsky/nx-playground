"""
Generate XNetwork CompositeResourceDefinition from schema.
Run: kcl run render/network_xrd.k > apis/xnetworks/definition.yaml
"""
import schemas.network as network
import schemas.xrd as xrd

_config = xrd.XRDConfig {
    name = network.NETWORK_XRD_NAME
    group = network.NETWORK_XRD_GROUP
    kind = network.NETWORK_XRD_KIND
    plural = network.NETWORK_XRD_PLURAL
    claimKind = network.NETWORK_XRD_CLAIM_KIND
    claimPlural = network.NETWORK_XRD_CLAIM_PLURAL
    compositionName = network.NETWORK_XRD_COMPOSITION
    specProperties = network.NETWORK_SPEC_PROPERTIES
    specRequired = network.NETWORK_SPEC_REQUIRED
    statusProperties = network.NETWORK_STATUS_PROPERTIES
}

[xrd.generate_xrd(_config)]
