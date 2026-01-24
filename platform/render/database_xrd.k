"""
Generate XDatabase CompositeResourceDefinition from schema.
Run: kcl run render/database_xrd.k > apis/xdatabases/definition.yaml
"""
import schemas.database as database
import schemas.xrd as xrd

_config = xrd.XRDConfig {
    name = database.DATABASE_XRD_NAME
    group = database.DATABASE_XRD_GROUP
    kind = database.DATABASE_XRD_KIND
    plural = database.DATABASE_XRD_PLURAL
    claimKind = database.DATABASE_XRD_CLAIM_KIND
    claimPlural = database.DATABASE_XRD_CLAIM_PLURAL
    compositionName = database.DATABASE_XRD_COMPOSITION
    specProperties = database.DATABASE_SPEC_PROPERTIES
    specRequired = database.DATABASE_SPEC_REQUIRED
    statusProperties = database.DATABASE_STATUS_PROPERTIES
}

[xrd.generate_xrd(_config)]
