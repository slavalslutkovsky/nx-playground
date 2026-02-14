"""
Multi-provider CI generation using modular KCL schemas.

This package generates CI configurations for multiple providers (GitHub Actions, Tekton)
from a single configuration source. Users can choose from pre-built presets or create
custom configurations.

Usage:
    # Generate GitHub workflow using preset
    kcl run main.k -e githubWorkflow

    # Generate Tekton resources using preset
    kcl run main.k -e tektonResources

    # Generate all for a specific config
    kcl run main.k -D config_file=./my-config.k

Presets available:
    - presets/rust_single.k     - Rust single application
    - presets/tanstack_app.k    - TanStack frontend app
    - presets/tauri_app.k       - Tauri desktop app
    - presets/go_service.k      - Go backend service
    - presets/nx_monorepo.k     - NX monorepo
"""

# Import providers
import .providers.github.main as github
import .providers.tekton.main as tekton

# Import presets
import .presets.nx_monorepo as nx_preset

# =============================================================================
# CURRENT REPO CONFIGURATION
# =============================================================================

# Using the nx_monorepo preset for this repo
config = nx_preset.nx_playground

# =============================================================================
# GENERATE CI FOR CONFIGURED PROVIDERS
# =============================================================================

# GitHub Actions workflow
githubWorkflow = github.generate_monorepo(config)

# Tekton resources (tasks + pipeline)
tektonResources = tekton.generate_monorepo(config)
# =============================================================================
# EXAMPLE: USING OTHER PRESETS
# =============================================================================
# Uncomment to generate for different app types:
# --- Rust single app ---
# import .presets.rust_single as rust_preset
# rust_config = rust_preset.default_config("my-rust-app")
# rust_github = github.generate_rust(rust_config)
# rust_tekton = tekton.generate_rust(rust_config)
# --- TanStack app ---
# import .presets.tanstack_app as tanstack_preset
# tanstack_config = tanstack_preset.default_config("my-tanstack-app")
# tanstack_github = github.generate_node(tanstack_config)
# tanstack_tekton = tekton.generate_node(tanstack_config)
# --- Tauri app ---
# import .presets.tauri_app as tauri_preset
# tauri_config = tauri_preset.default_config("my-tauri-app")
# tauri_github = github.generate_tauri(tauri_config)
# --- Go service ---
# import .presets.go_service as go_preset
# go_config = go_preset.default_config("my-go-api")
# go_github = github.generate_go(go_config)
# go_tekton = tekton.generate_go(go_config)
#apiVersion: security.istio.io/v1
 #  kind: AuthorizationPolicy
 #  metadata:
  #   name: allow-external-to-gateway
  #   namespace: gateway
  # spec:
  #   selector:
  #     matchLabels:
  #       istio.io/gateway-name: main-gateway
  #   action: ALLOW
  #   rules:
  #   - {}
