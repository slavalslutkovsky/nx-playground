"""
Multi-provider CI generation using official KCL schemas.

Usage:
    kcl run main.k                            # Output all
    kcl run main.k -e githubWorkflow          # Output GitHub only
    kcl run main.k -e tektonResources         # Output Tekton only
"""
import github_workflow as gh
import k8s.apimachinery.pkg.apis.meta.v1 as metav1

# =============================================================================
# TEKTON SCHEMAS (Pipeline package only has PipelineRun/TaskRun)
# =============================================================================

schema TektonTask:
    """Tekton Task resource"""
    apiVersion: str = "tekton.dev/v1"
    kind: str = "Task"
    metadata: metav1.ObjectMeta
    spec: TektonTaskSpec

schema TektonTaskSpec:
    description?: str
    params?: [TektonParam]
    steps: [TektonStep]
    workspaces?: [TektonWorkspaceDeclaration]

schema TektonParam:
    name: str
    $type?: str = "string"
    description?: str
    default?: str

schema TektonStep:
    name: str
    image: str
    script?: str
    command?: [str]
    args?: [str]
    env?: [TektonEnvVar]
    workingDir?: str

schema TektonEnvVar:
    name: str
    value?: str
    valueFrom?: any

schema TektonWorkspaceDeclaration:
    name: str
    description?: str
    mountPath?: str

schema TektonPipeline:
    """Tekton Pipeline resource"""
    apiVersion: str = "tekton.dev/v1"
    kind: str = "Pipeline"
    metadata: metav1.ObjectMeta
    spec: TektonPipelineSpec

schema TektonPipelineSpec:
    description?: str
    params?: [TektonParam]
    tasks: [TektonPipelineTask]
    workspaces?: [TektonPipelineWorkspace]

schema TektonPipelineTask:
    name: str
    taskRef?: TektonTaskRef
    runAfter?: [str]
    params?: [TektonPipelineTaskParam]
    workspaces?: [TektonPipelineTaskWorkspace]

schema TektonTaskRef:
    name: str

schema TektonPipelineTaskParam:
    name: str
    value: str

schema TektonPipelineTaskWorkspace:
    name: str
    workspace: str

schema TektonPipelineWorkspace:
    name: str

# =============================================================================
# REPO CONFIGURATION
# =============================================================================

schema RepoConfig:
    """Repository configuration - provider agnostic"""
    name: str
    $type: "monorepo" | "polyrepo" = "monorepo"
    monorepo_tool: "nx" | "turborepo" | "moonrepo" | "none" = "nx"
    package_manager: "bun" | "npm" | "pnpm" | "yarn" = "bun"
    languages: [str] = ["typescript"]
    main_branch: str = "main"
    use_mise: bool = True
    use_sccache: bool = False
    sccache_bucket: str = ""

# This repo's config
config = RepoConfig {
    name = "nx-playground"
    $type = "monorepo"
    monorepo_tool = "nx"
    package_manager = "bun"
    languages = ["typescript", "rust", "go", "kcl"]
    use_mise = False  # Disabled in CI - use targeted setup actions instead (saves ~360MB cache)
    use_sccache = True
    sccache_bucket = "sccache-bucket"
}

# =============================================================================
# REUSABLE STEP BUILDERS (using dicts to avoid schema defaults)
# =============================================================================

checkoutStep = lambda fetch_depth: int = 0 -> {str:} {
    {
        name = "Checkout"
        uses = "actions/checkout@v4"
        "with" = {"fetch-depth" = fetch_depth}
    }
}

miseStep = lambda working_dir: str = "", cache_prefix: str = "mise" -> {str:} {
    {
        name = "Setup mise"
        uses = "jdx/mise-action@v2"
        "with" = {
            "experimental" = "true"
            "cache" = "true"
            "cache_key_prefix" = "${cache_prefix}-\${{ runner.os }}"
        } | ({"working_directory" = working_dir} if working_dir else {})
    }
}

bunStep = lambda -> {str:} {
    {
        name = "Setup Bun"
        uses = "oven/setup-bun@v2"
    }
}

nodeStep = lambda version: str = "24" -> {str:} {
    {
        name = "Setup Node.js"
        uses = "actions/setup-node@v4"
        "with" = {
            "node-version" = version
        }
    }
}

goStep = lambda version: str = "1.23" -> {str:} {
    {
        name = "Setup Go"
        uses = "actions/setup-go@v5"
        "with" = {
            "go-version" = version
            "cache" = "true"
        }
    }
}

nxShasStep = lambda -> {str:} {
    {
        name = "Set NX SHAs"
        uses = "nrwl/nx-set-shas@v4"
    }
}

installStep = lambda pm: str = "bun" -> {str:} {
    _cmd = {
        bun = "bun install"
        npm = "npm ci"
        pnpm = "pnpm install --frozen-lockfile"
        yarn = "yarn install --frozen-lockfile"
    }[pm]
    {
        name = "Install dependencies"
        run = _cmd
    }
}

nxAffectedStep = lambda target: str, pm: str = "bun" -> {str:} {
    {
        name = "Run ${target}"
        run = "${pm} nx affected -t ${target}"
    }
}

rustupStep = lambda -> {str:} {
    {
        name = "Install Rust toolchain"
        run = "rustup toolchain install stable --profile minimal"
    }
}

gcpAuthStep = lambda -> {str:} {
    {
        name = "Authenticate to Google Cloud"
        uses = "google-github-actions/auth@v3"
        "with" = {
            "credentials_json" = "\${{ secrets.GCP_SA_KEY }}"
        }
    }
}

sccacheStep = lambda -> {str:} {
    {
        name = "Setup sccache"
        uses = "mozilla-actions/sccache-action@v0.0.9"
    }
}

# =============================================================================
# COMMON STEP SEQUENCES
# =============================================================================

baseSteps = lambda cfg: RepoConfig -> [{str:}] {
    [
        checkoutStep(0)
        nxShasStep() if cfg.monorepo_tool == "nx" else None
        # sccache must come BEFORE tool setup so cargo tools use it
        rustupStep() if cfg.use_sccache else None
        gcpAuthStep() if cfg.use_sccache else None
        sccacheStep() if cfg.use_sccache else None
        # Use mise OR targeted setup actions (bun/node/go)
        miseStep() if cfg.use_mise else None
        bunStep() if not cfg.use_mise and cfg.package_manager == "bun" else None
        nodeStep() if not cfg.use_mise and "typescript" in cfg.languages else None
        goStep() if not cfg.use_mise and "go" in cfg.languages else None
        installStep(cfg.package_manager)
    ]
}

# =============================================================================
# GITHUB ACTIONS WORKFLOW
# =============================================================================

githubWorkflow = gh.GithubWorkflow {
    name = "CI"

    on = {
        push = {branches = [config.main_branch]}
        pull_request = {}
    }

    env = {
        NX_CLOUD_ACCESS_TOKEN = "\${{ secrets.NX_CLOUD_ACCESS_TOKEN }}"
    } | ({
        CARGO_INCREMENTAL = "0"
        RUSTC_WRAPPER = "sccache"
        SCCACHE_GCS_BUCKET = "\${{ vars.SCCACHE_GCS_BUCKET }}"
        SCCACHE_GCS_RW_MODE = "READ_WRITE"
        SCCACHE_GCS_KEY_PREFIX = "sccache"
    } if config.use_sccache else {})

    permissions = {
        actions = "read"
        contents = "read"
    }

    jobs = {
        lint = {
            name = "Lint"
            "runs-on" = "ubuntu-latest"
            steps = [s for s in baseSteps(config) if s] + [
                nxAffectedStep("lint", config.package_manager)
            ]
        }
        test = {
            name = "Test"
            "runs-on" = "ubuntu-latest"
            steps = [s for s in baseSteps(config) if s] + [
                nxAffectedStep("test", config.package_manager)
            ]
        }
        build = {
            name = "Build"
            "runs-on" = "ubuntu-latest"
            needs = ["lint", "test"]
            steps = [s for s in baseSteps(config) if s] + [
                nxAffectedStep("build", config.package_manager)
            ]
        }
    }
    permissions = {
        actions = "read"
        contents = "read"
    }
}

# =============================================================================
# TEKTON RESOURCES
# =============================================================================

# Base image for Node/Bun tasks
_nodeImage = "oven/bun:1"
_nxImage = "node:22-slim"

# Tekton Task: Clone repository
tektonCloneTask = TektonTask {
    metadata = metav1.ObjectMeta {
        name = "git-clone"
    }
    spec = TektonTaskSpec {
        description = "Clone the repository"
        workspaces = [
            TektonWorkspaceDeclaration {name = "source", description = "Workspace to clone into"}
        ]
        params = [
            TektonParam {name = "repo-url", description = "Git repository URL"}
            TektonParam {name = "revision", default = "main", description = "Git revision"}
        ]
        steps = [
            TektonStep {
                name = "clone"
                image = "alpine/git:latest"
                script = """
                    git clone $(params.repo-url) $(workspaces.source.path)
                    cd $(workspaces.source.path)
                    git checkout $(params.revision)
                """
            }
        ]
    }
}

# Tekton Task: Install dependencies
tektonInstallTask = TektonTask {
    metadata = metav1.ObjectMeta {
        name = "install-deps"
    }
    spec = TektonTaskSpec {
        description = "Install project dependencies"
        workspaces = [
            TektonWorkspaceDeclaration {name = "source"}
        ]
        steps = [
            TektonStep {
                name = "install"
                image = _nodeImage
                workingDir = "$(workspaces.source.path)"
                script = """
                    ${config.package_manager} install
                """
            }
        ]
    }
}

# Tekton Task: Run NX target
tektonNxTask = lambda target: str -> TektonTask {
    TektonTask {
        metadata = metav1.ObjectMeta {
            name = "nx-${target}"
        }
        spec = TektonTaskSpec {
            description = "Run nx ${target}"
            workspaces = [
                TektonWorkspaceDeclaration {name = "source"}
            ]
            params = [
                TektonParam {name = "affected", type = "string", default = "true"}
            ]
            steps = [
                TektonStep {
                    name = "nx-${target}"
                    image = _nodeImage
                    workingDir = "$(workspaces.source.path)"
                    script = """
                        if [ "$(params.affected)" = "true" ]; then
                            ${config.package_manager} nx affected -t ${target}
                        else
                            ${config.package_manager} nx run-many -t ${target}
                        fi
                    """
                }
            ]
        }
    }
}

# Generate tasks for each target
tektonLintTask = tektonNxTask("lint")
tektonTestTask = tektonNxTask("test")
tektonBuildTask = tektonNxTask("build")

# Tekton Pipeline: Full CI
tektonPipeline = TektonPipeline {
    metadata = metav1.ObjectMeta {
        name = "${config.name}-ci"
    }
    spec = TektonPipelineSpec {
        description = "CI pipeline for ${config.name}"
        params = [
            TektonParam {name = "repo-url", description = "Git repository URL"}
            TektonParam {name = "revision", default = "main"}
        ]
        workspaces = [
            TektonPipelineWorkspace {name = "shared-workspace"}
        ]
        tasks = [
            TektonPipelineTask {
                name = "clone"
                taskRef = TektonTaskRef {name = "git-clone"}
                params = [
                    TektonPipelineTaskParam {name = "repo-url", value = "$(params.repo-url)"}
                    TektonPipelineTaskParam {name = "revision", value = "$(params.revision)"}
                ]
                workspaces = [
                    TektonPipelineTaskWorkspace {name = "source", workspace = "shared-workspace"}
                ]
            }
            TektonPipelineTask {
                name = "install"
                taskRef = TektonTaskRef {name = "install-deps"}
                runAfter = ["clone"]
                workspaces = [
                    TektonPipelineTaskWorkspace {name = "source", workspace = "shared-workspace"}
                ]
            }
            TektonPipelineTask {
                name = "lint"
                taskRef = TektonTaskRef {name = "nx-lint"}
                runAfter = ["install"]
                workspaces = [
                    TektonPipelineTaskWorkspace {name = "source", workspace = "shared-workspace"}
                ]
            }
            TektonPipelineTask {
                name = "test"
                taskRef = TektonTaskRef {name = "nx-test"}
                runAfter = ["install"]
                workspaces = [
                    TektonPipelineTaskWorkspace {name = "source", workspace = "shared-workspace"}
                ]
            }
            TektonPipelineTask {
                name = "build"
                taskRef = TektonTaskRef {name = "nx-build"}
                runAfter = ["lint", "test"]
                workspaces = [
                    TektonPipelineTaskWorkspace {name = "source", workspace = "shared-workspace"}
                ]
            }
        ]
    }
}

# All Tekton resources bundled
tektonResources = [
    tektonCloneTask
    tektonInstallTask
    tektonLintTask
    tektonTestTask
    tektonBuildTask
    tektonPipeline
]
