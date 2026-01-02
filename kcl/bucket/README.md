# bucket

This project was initialized using the `project-template-scratch` example (version
0.1.0). Update this README with information about what the
project contains and an example of how to use it.

## Installation
up example generate \
--api-group platform.example.com \
--api-version v1alpha1 \
--kind WebApp\
--name my-app \
--scope namespace \
--namespace default

up xrd generate examples/webapp/my-app.yaml
up composition generate apis/webapps/definition.yaml
up function generate --language=kcl compose-resources apis/webapps/composition.yaml
up dependency add --api k8s:v1.33.0


## Examples
kubectl apply -f examples/webapp/my-app.yaml
up dep add xpkg.upbound.io/upbound/provider-aws-ecr:v2.3.0
up dep add xpkg.upbound.io/upbound/provider-gcp-artifact:v2.4.0
up dep add xpkg.upbound.io/crossplane-contrib/function-cue:v0.4.1
up dep add xpkg.upbound.io/upbound/function-claude:v0.4.0
up dep add xpkg.upbound.io/crossplane-contrib/function-kcl:v0.12.0
up dep add xpkg.upbound.io/upbound/provider-aws-ses:latest
up dep add xpkg.upbound.io/upbound/function-claude:v0.2.0
up dep add xpkg.upbound.io/upbound/function-analysis-gate:v0.0.0-20250808233445-b3bb3dafbd25
up dep add xpkg.upbound.io/upbound/function-remediation-gate:v0.0.0-20250808233532-ad1d6ad2aea6
up dep add xpkg.upbound.io/upbound/function-event-filter:v0.0.0-20250808235120-d07a570f15d6

## Testing

TODO
