"""
Generate XDatabase CompositeResourceDefinition from schema.
Run: kcl run render/database_xrd.k > apis/xdatabases/definition.yaml
"""
import schemas.database as database
import schemas.xrd as xrd

_config = xrd.XRDConfig {
    name = database.XRD_NAME
    group = database.XRD_GROUP
    kind = database.XRD_KIND
    plural = database.XRD_PLURAL
    claimKind = database.XRD_CLAIM_KIND
    claimPlural = database.XRD_CLAIM_PLURAL
    compositionName = database.XRD_COMPOSITION
    specProperties = database.SPEC_PROPERTIES
    specRequired = database.SPEC_REQUIRED
    statusProperties = database.STATUS_PROPERTIES
}

[xrd.generate_xrd(_config)]
