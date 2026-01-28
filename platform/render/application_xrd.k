"""
Generate XApplication CompositeResourceDefinition from schema.
Run: kcl run render/application_xrd.k > apis/xapplications/definition.yaml
"""
import schemas.application as application
import schemas.xrd as xrd

_config = xrd.XRDConfig {
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
}

[xrd.generate_xrd(_config)]
