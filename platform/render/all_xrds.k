"""
Generate all XRDs as a list.
Run: kcl run render/all_xrds.k -o yaml
This outputs all XRDs in a single YAML stream (useful for debugging).
"""
import schemas.bucket as bucket
import schemas.database as database
import schemas.registry as registry
import schemas.application as application
import schemas.network as network
import schemas.xrd as xrd

_bucket_xrd = xrd.generate_xrd(xrd.XRDConfig {
    name = bucket.XRD_NAME
    group = bucket.XRD_GROUP
    kind = bucket.XRD_KIND
    plural = bucket.XRD_PLURAL
    claimKind = bucket.XRD_CLAIM_KIND
    claimPlural = bucket.XRD_CLAIM_PLURAL
    compositionName = bucket.XRD_COMPOSITION
    specProperties = bucket.SPEC_PROPERTIES
    specRequired = bucket.SPEC_REQUIRED
    statusProperties = bucket.STATUS_PROPERTIES
})

_database_xrd = xrd.generate_xrd(xrd.XRDConfig {
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
})

_registry_xrd = xrd.generate_xrd(xrd.XRDConfig {
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
})

_application_xrd = xrd.generate_xrd(xrd.XRDConfig {
    name = application.XRD_NAME
    group = application.XRD_GROUP
    kind = application.XRD_KIND
    plural = application.XRD_PLURAL
    claimKind = application.XRD_CLAIM_KIND
    claimPlural = application.XRD_CLAIM_PLURAL
    compositionName = application.XRD_COMPOSITION
    specProperties = application.SPEC_PROPERTIES
    specRequired = application.SPEC_REQUIRED
    statusProperties = application.STATUS_PROPERTIES
})

_network_xrd = xrd.generate_xrd(xrd.XRDConfig {
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
})

[_bucket_xrd, _database_xrd, _registry_xrd, _application_xrd, _network_xrd]
