"""
Generate XBucket CompositeResourceDefinition from schema.
Run: kcl run render/bucket_xrd.k > apis/xbuckets/definition.yaml
"""
import schemas.bucket as bucket
import schemas.xrd as xrd

_config = xrd.XRDConfig {
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
    additionalPrinterColumns = bucket.ADDITIONAL_PRINTER_COLUMNS
}

[xrd.generate_xrd(_config)]




