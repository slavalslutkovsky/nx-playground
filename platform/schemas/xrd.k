"""
XRD (CompositeResourceDefinition) generation helpers.
Creates Crossplane XRD YAML from KCL schema metadata.
"""

schema PrinterColumn:
    """Additional printer column for kubectl get output."""
    name: str
    $type: str = "string"
    jsonPath: str

schema XRDConfig:
    """Configuration for generating a CompositeResourceDefinition."""
    name: str
    group: str
    kind: str
    plural: str
    claimKind: str
    claimPlural: str
    compositionName: str
    specProperties: {str:any}
    specRequired: [str]
    statusProperties: {str:any}
    additionalPrinterColumns?: [PrinterColumn]
    version: str = "v1alpha1"

generate_xrd = lambda config: XRDConfig -> {str:any} {
    """Generate a CompositeResourceDefinition from config."""
    {
        apiVersion = "apiextensions.crossplane.io/v1"
        kind = "CompositeResourceDefinition"
        metadata = {
            name = config.name
        }
        spec = {
            group = config.group
            names = {
                kind = config.kind
                plural = config.plural
            }
            claimNames = {
                kind = config.claimKind
                plural = config.claimPlural
            }
            defaultCompositionRef = {
                name = config.compositionName
            }
            versions = [{
                name = config.version
                served = True
                referenceable = True
                $schema = {
                    openAPIV3Schema = {
                        $type = "object"
                        properties = {
                            spec = {
                                $type = "object"
                                required = config.specRequired
                                properties = config.specProperties
                            }
                            status = {
                                $type = "object"
                                properties = config.statusProperties
                            }
                        }
                    }
                }
                additionalPrinterColumns = [
                    {
                        name = col.name
                        $type = col.$type
                        jsonPath = col.jsonPath
                    }
                    for col in config.additionalPrinterColumns
                ] if config.additionalPrinterColumns else None
            }]
        }
    }
}
