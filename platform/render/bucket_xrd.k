"""
Generate XBucket CompositeResourceDefinition from schema.
Run: kcl run render/bucket_xrd.k > apis/xbuckets/definition.yaml
"""
import schemas.bucket as bucket
import schemas.xrd as xrd

_config = xrd.XRDConfig {
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
    additionalPrinterColumns = bucket.ADDITIONAL_PRINTER_COLUMNS
}

[xrd.generate_xrd(_config)]




