[package]
name = "compose-bucket"
version = "0.0.1"

[dependencies]
# Models bundled with the package (copied from .up/kcl/models during publish)
models = { path = "./model" }
# Schemas from OCI registry
schemas = { oci = "oci://docker.io/yurikrupnik/platform-schemas", tag = "0.0.1" }
