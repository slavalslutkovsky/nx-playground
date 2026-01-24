import models.io.upbound.dev.meta.v1alpha1 as metav1alpha1

_items = [
    metav1alpha1.CompositionTest{
        metadata.name = "test-xdatabase-gcp"
        spec = {
            compositionPath = "apis/xdatabases/composition.yaml"
            xrdPath = "apis/xdatabases/definition.yaml"
            xrPath = "tests/test-xdatabase/xr-gcp.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
    metav1alpha1.CompositionTest{
        metadata.name = "test-xdatabase-aws"
        spec = {
            compositionPath = "apis/xdatabases/composition.yaml"
            xrdPath = "apis/xdatabases/definition.yaml"
            xrPath = "tests/test-xdatabase/xr-aws.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
    metav1alpha1.CompositionTest{
        metadata.name = "test-xdatabase-k8s"
        spec = {
            compositionPath = "apis/xdatabases/composition.yaml"
            xrdPath = "apis/xdatabases/definition.yaml"
            xrPath = "tests/test-xdatabase/xr-k8s.yaml"
            timeoutSeconds = 60
            validate = False
        }
    }
]
items = _items
