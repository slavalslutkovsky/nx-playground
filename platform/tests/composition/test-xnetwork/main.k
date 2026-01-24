"""
XNetwork composition tests - AWS and GCP providers.

Run with:
    up test run tests/composition/test-xnetwork
"""
import models.io.upbound.dev.meta.v1alpha1 as metav1alpha1

_items = [
    metav1alpha1.CompositionTest{
        metadata.name = "test-xnetwork-aws"
        spec = {
            compositionPath = "apis/xnetworks/composition.yaml"
            xrdPath = "apis/xnetworks/definition.yaml"
            xrPath = "tests/composition/test-xnetwork/xr-aws.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
    metav1alpha1.CompositionTest{
        metadata.name = "test-xnetwork-gcp"
        spec = {
            compositionPath = "apis/xnetworks/composition.yaml"
            xrdPath = "apis/xnetworks/definition.yaml"
            xrPath = "tests/composition/test-xnetwork/xr-gcp.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
]
items = _items
