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
    name = bucket.BUCKET_XRD_NAME
    group = bucket.BUCKET_XRD_GROUP
    kind = bucket.BUCKET_XRD_KIND
    plural = bucket.BUCKET_XRD_PLURAL
    claimKind = bucket.BUCKET_XRD_CLAIM_KIND
    claimPlural = bucket.BUCKET_XRD_CLAIM_PLURAL
    compositionName = bucket.BUCKET_XRD_COMPOSITION
    specProperties = bucket.BUCKET_SPEC_PROPERTIES
    specRequired = bucket.BUCKET_SPEC_REQUIRED
    statusProperties = bucket.BUCKET_STATUS_PROPERTIES
})

_database_xrd = xrd.generate_xrd(xrd.XRDConfig {
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
})

_registry_xrd = xrd.generate_xrd(xrd.XRDConfig {
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
})

_application_xrd = xrd.generate_xrd(xrd.XRDConfig {
    name = application.APPLICATION_XRD_NAME
    group = application.APPLICATION_XRD_GROUP
    kind = application.APPLICATION_XRD_KIND
    plural = application.APPLICATION_XRD_PLURAL
    claimKind = application.APPLICATION_XRD_CLAIM_KIND
    claimPlural = application.APPLICATION_XRD_CLAIM_PLURAL
    compositionName = application.APPLICATION_XRD_COMPOSITION
    specProperties = application.APPLICATION_SPEC_PROPERTIES
    specRequired = application.APPLICATION_SPEC_REQUIRED
    statusProperties = application.APPLICATION_STATUS_PROPERTIES
})

_network_xrd = xrd.generate_xrd(xrd.XRDConfig {
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
})

[_bucket_xrd, _database_xrd, _registry_xrd, _application_xrd, _network_xrd]
