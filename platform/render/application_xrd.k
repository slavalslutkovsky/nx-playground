"""
Generate XApplication CompositeResourceDefinition from schema.
Run: kcl run render/application_xrd.k > apis/xapplications/definition.yaml
"""
import schemas.application as application
import schemas.xrd as xrd

_config = xrd.XRDConfig {
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
}

[xrd.generate_xrd(_config)]
