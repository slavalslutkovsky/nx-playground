[package]
name = "compose-database"
version = "0.0.1"

[dependencies]
crossplane-provider-upjet-aws = "1.23.0"
models = { path = "./model" }
# TODO: Republish OCI package with helpers.k, regions.k, mappings.k
schemas = { path = "../../schemas" }
cloudnative-pg = "1.27.0"
