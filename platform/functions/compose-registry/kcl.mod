[package]
name = "compose-registry"
version = "0.0.1"

[dependencies]
crossplane = "v2.0.2"
crossplane-provider-upjet-aws = "1.23.0"
crossplane-provider-upjet-gcp = "1.0.5"
harbor-operator = "0.2.1"
models = { path = "./model" }
schemas = { path = "../../schemas" }
