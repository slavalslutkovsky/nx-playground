"""
XBucket composition tests - AWS and GCP providers.

Run with:
    up test run tests/composition/test-xbucket
"""
import models.io.upbound.dev.meta.v1alpha1 as metav1alpha1

_items = [
    metav1alpha1.CompositionTest{
        metadata.name = "test-xbucket-aws"
        spec = {
            compositionPath = "apis/xbuckets/composition.yaml"
            xrdPath = "apis/xbuckets/definition.yaml"
            xrPath = "tests/composition/test-xbucket/xr-aws.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
    metav1alpha1.CompositionTest{
        metadata.name = "test-xbucket-gcp"
        spec = {
            compositionPath = "apis/xbuckets/composition.yaml"
            xrdPath = "apis/xbuckets/definition.yaml"
            xrPath = "tests/composition/test-xbucket/xr-gcp.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
]
items = _items
