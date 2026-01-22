import models.io.upbound.dev.meta.v1alpha1 as metav1alpha1

_items = [
    metav1alpha1.CompositionTest{
        metadata.name = "test-xapplication-gcp"
        spec = {
            compositionPath = "apis/xapplications/composition.yaml"
            xrdPath = "apis/xapplications/definition.yaml"
            xrPath = "tests/test-xapplication/xr-gcp.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
    metav1alpha1.CompositionTest{
        metadata.name = "test-xapplication-aws"
        spec = {
            compositionPath = "apis/xapplications/composition.yaml"
            xrdPath = "apis/xapplications/definition.yaml"
            xrPath = "tests/test-xapplication/xr-aws.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
]
items = _items
