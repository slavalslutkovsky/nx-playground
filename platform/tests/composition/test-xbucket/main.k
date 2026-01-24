import models.io.upbound.dev.meta.v1alpha1 as metav1alpha1

_items = [
    metav1alpha1.CompositionTest{
        metadata.name = "test-xbucket-gcp"
        spec = {
            compositionPath = "apis/xbuckets/composition.yaml"
            xrdPath = "apis/xbuckets/definition.yaml"
            xrPath = "tests/test-xbucket/xr.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
]
items = _items
