
bundle: {
//	  _id:   string @timoni(runtime:string:ACCOUNT_ID)
//    _reg:  string @timoni(runtime:string:REGION)
//    _pass: string @timoni(runtime:string:REDIS_PASS)
//    _shit: string @timoni(runtime:string:SHIT)
//    _cluster: string @timoni(runtime:string:TIMONI_CLUSTER_NAME)
//    _env:     string @timoni(runtime:string:TIMONI_CLUSTER_GROUP)
    apiVersion: "v1alpha1"
    name:       "podinfo"
    instances: {
//		  if _env == "development" {
////				replicas: 1
//		  }
//    	app: {
//				module: url: "oci://ghcr.io/stefanprodan/modules/podinfo"
//				namespace: "apps"
//				values: {
//						ui: message: "Hosted by \(_cluster)"
//						if _env == "development" {
//								replicas: 1
//						}
//						if _env == "staging" {
//								replicas: 2
//						}
//						if _env == "production" {
//								replicas: 3
//						}
//				}
//		  }
//    	flux: {
//				module: url: "oci://ghcr.io/stefanprodan/modules/flux-aio"
//				namespace: "flux-system"
//				values: {
//						controllers: {
//								helm: enabled:         true
//								kustomize: enabled:    true
//								notification: enabled: true
//						}
//						hostNetwork:     false
//				  	securityProfile: "privileged"
//				}
//		  }
//		  prometheus: {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//			  namespace: "prometheus"
//		 	  values: {
//		 	  	repository: url: "https://prometheus-community.github.io/helm-charts"
//					chart: {
//						name:    "kube-prometheus-stack"
//				 		version: "60.1.*"
//					}
//					helmValues: {
//						installCRDs: true
//					}
//				}
//		  }
//		  kwasm: {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//			  namespace: "kwasm"
//		 	  values: {
//		 	  	repository: url: "http://kwasm.sh/kwasm-operator/"
//					chart: {
//						name:    "kwasm-operator"
//				 		version: "0.2.*"
//					}
//					helmValues: {
//						installCRDs: true
//					}
//				}
//		  }
//		  "external-secrets": {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//			  namespace: "external-secrets"
//		 	  values: {
//		 	  	repository: url: "https://charts.external-secrets.io"
//					chart: {
//						name:    "external-secrets"
//				 		version: "0.9.*"
//					}
//					helmValues: {
//						installCRDs: true
//					}
//				}
//		  }
//		  "minio": {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//			  namespace: "minio-official"
//		 	  values: {
//		 	  	repository: url: "https://charts.min.io"
//					chart: {
//						name:    "minio-official"
//				 		version: "5.3.*"
//					}
//					helmValues: {
//						installCRDs: true
//					}
//				}
//			}
//		  crossplane: {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//				namespace: "crossplane"
//				values: {
//					repository: url: "https://charts.crossplane.io/stable"
//					chart: {
//						name:    "crossplane"
//					  version: "1.15.*"
//					}
//					helmValues: {
//						installCRDs: true
//					}
//				}
//			}
//			komoplane: {
//				module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//				namespace: "keda"
//				values: {
//					repository: url: "https://kedacore.github.io/charts"
//					chart: {
//						name:    "keda"
//				  	version: "2.14.*"
//			 	  }
//				  helmValues: {}
//		 		}
//		  }
//		  "cert-manager": {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//				namespace: "cert-manager"
//				values: {
//					repository: url: "https://charts.jetstack.io"
//					chart: {
//						name:    "cert-manager"
//				  	version: "1.16.*"
//			 	  }
//				  helmValues: {}
//		 		}
//		  }
//		  "argo": {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//				namespace: "argo"
//				values: {
//					repository: url: "https://argoproj.github.io/argo-helm"
//					chart: {
//						name:    "argo"
//				  	version: "7.7.*"
//			 	  }
//				  helmValues: {}
//		 		}
//		  }
//kubeshark: {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//			  namespace: "kubeshark"
//		 	  values: {
//		 	  	repository: url: "https://helm.kubeshark.co/"
//					chart: {
//						name:    "kubeshark"
//				 		version: "52.3.*"
//					}
//					helmValues: {
////						installCRDs: true
//					}
//				}
//		  }
//		  flagger: {
//		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
//			  namespace: "istio-system"
//		 	  values: {
//		 	  	repository: url: "https://flagger.app"
//					chart: {
//						name:    "flagger"
//				 		version: "1.37.*"
//					}
//					helmValues: {
//						installCRDs: true
//					}
//				}
//		  }
		  testkube: {
		  	module: url: "oci://ghcr.io/stefanprodan/modules/flux-helm-release"
		  	namespace: "flux-system"
		  	values: {
		  		repository: url: "https://kubeshop.github.io/helm-charts"
//		  		targetNamespace: testkube
		  		chart: {
		  			name: "testkube"
		  			version: "2.0.x"
		  		}
//		  		install: {
//		  			createNamespace: true
//		  		}
		  		helmValues: {
            installCRDs: true
				  	"testkube-api.cloud.key": "tkcagnt_3f99186b2d07881577956aba5df049}"
				  	"testkube-api.cloud.orgId": "tkcorg_9f5a2231a41b2b50"
				  	 "testkube-api.cloud.envId": "tkcenv_c72da0c2e1792d2c"
				  	 "testkube-logs.pro.key": "tkcagnt_3f99186b2d07881577956aba5df049"
				  	 "testkube-logs.pro.orgId": "tkcorg_9f5a2231a41b2b50"
				  	 "testkube-logs.pro.envId": "tkcenv_c72da0c2e1792d2c"
				  	 "testkube-api.minio.enabled": false
				  	 "mongodb.enabled": false
				  	 "testkube-dashboard.enabled": false
				  	 "testkube-api.cloud.url": "agent.testkube.io:443"
				  	 "testkube-logs-service.pro.url": "logs.testkube.io:443"
				  }
		  	}
		  }
//			"storage": {
//				module: url: "oci://me-west1-docker.pkg.dev/devops-386509/platform/storage"
//				namespace: "apps"
//				values: {
////					image: repository: "yurikrupnik/solid-app"
////					image: digest: ""
////					image: tag: "latest"
//			  }
//		  }
//		  "be-app": {
//				module: url: "oci://me-west1-docker.pkg.dev/devops-386509/platform/be-app"
//				namespace: "apps"
//				values: {
////					imagePullSecrets: [{
////						name: "docker-registry-secret"
////					}]
//					image: repository: "me-west1-docker.pkg.dev/devops-386509/platform/web-playground"
//					image: digest: ""
////					image: tag: "pr-3"
//					image: tag: "my-shit1"
////					configMapRef: "aws-info"
////          secretRef: "my-secret"
//
////					env: {
////            ENV_VAR_NAME: "value"
////            ANOTHER_VAR: "another_value"
////        	}
//			  }
//		  }
//		  "fe-app": {
//				module: url: "oci://me-west1-docker.pkg.dev/devops-386509/platform/fe-app"
//				namespace: "apps"
//				values: {
//					image: repository: "yurikrupnik/solid-app"
//					image: digest: ""
//					image: tag: "latest"
//			  }
//		  }
    }
}
