"""
XRegistry composition tests - AWS and GCP providers.

Run with:
    up test run tests/composition/test-xregistry
"""
import models.io.upbound.dev.meta.v1alpha1 as metav1alpha1

_items = [
    metav1alpha1.CompositionTest{
        metadata.name = "test-xregistry-aws"
        spec = {
            compositionPath = "apis/xregistries/composition.yaml"
            xrdPath = "apis/xregistries/definition.yaml"
            xrPath = "tests/composition/test-xregistry/xr-aws.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
    metav1alpha1.CompositionTest{
        metadata.name = "test-xregistry-gcp"
        spec = {
            compositionPath = "apis/xregistries/composition.yaml"
            xrdPath = "apis/xregistries/definition.yaml"
            xrPath = "tests/composition/test-xregistry/xr-gcp.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
]
items = _items
