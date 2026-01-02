/**
 * NX Playground Dagger Module
 *
 * Provides CI/CD pipelines for:
 * - Building Rust applications
 * - Running tests
 * - Provisioning databases (vector/graph)
 * - Publishing container images
 */

import {
  dag,
  Container,
  Directory,
  Service,
  object,
  func,
  field,
} from "@dagger.io/dagger";

@object()
export class NxPlayground {
  /**
   * Source directory
   */
  @field()
  source: Directory;

  constructor(source?: Directory) {
    this.source = source ?? dag.currentModule().source();
  }

  // ============================================================
  // DATABASE SERVICES
  // ============================================================

  /**
   * Start Qdrant vector database service
   */
  @func()
  qdrant(): Service {
    return dag
      .container()
      .from("qdrant/qdrant:latest")
      .withExposedPort(6333)
      .withExposedPort(6334)
      .withEnvVariable("QDRANT__LOG_LEVEL", "INFO")
      .asService();
  }

  /**
   * Start Neo4j graph database service
   */
  @func()
  neo4j(password: string = "password123"): Service {
    return dag
      .container()
      .from("neo4j:5-community")
      .withExposedPort(7474)
      .withExposedPort(7687)
      .withEnvVariable("NEO4J_AUTH", `neo4j/${password}`)
      .withEnvVariable("NEO4J_PLUGINS", '["apoc"]')
      .asService();
  }

  /**
   * Start ArangoDB multimodel database service
   */
  @func()
  arangodb(password: string = "rootpassword"): Service {
    return dag
      .container()
      .from("arangodb:3.11")
      .withExposedPort(8529)
      .withEnvVariable("ARANGO_ROOT_PASSWORD", password)
      .asService();
  }

  /**
   * Start Milvus vector database service (standalone mode)
   */
  @func()
  milvus(): Service {
    const etcd = dag
      .container()
      .from("quay.io/coreos/etcd:v3.5.5")
      .withEnvVariable("ETCD_AUTO_COMPACTION_MODE", "revision")
      .withEnvVariable("ETCD_AUTO_COMPACTION_RETENTION", "1000")
      .withEnvVariable("ETCD_QUOTA_BACKEND_BYTES", "4294967296")
      .withExec([
        "etcd",
        "-advertise-client-urls=http://127.0.0.1:2379",
        "-listen-client-urls=http://0.0.0.0:2379",
      ])
      .asService();

    const minio = dag
      .container()
      .from("minio/minio:RELEASE.2023-03-20T20-16-18Z")
      .withEnvVariable("MINIO_ACCESS_KEY", "minioadmin")
      .withEnvVariable("MINIO_SECRET_KEY", "minioadmin")
      .withExec(["minio", "server", "/minio_data"])
      .asService();

    return dag
      .container()
      .from("milvusdb/milvus:v2.3.3")
      .withServiceBinding("etcd", etcd)
      .withServiceBinding("minio", minio)
      .withEnvVariable("ETCD_ENDPOINTS", "etcd:2379")
      .withEnvVariable("MINIO_ADDRESS", "minio:9000")
      .withExposedPort(19530)
      .withExposedPort(9091)
      .withExec(["milvus", "run", "standalone"])
      .asService();
  }

  /**
   * Start a PostgreSQL database service
   */
  @func()
  postgres(
    database: string = "mydatabase",
    user: string = "myuser",
    password: string = "mypassword"
  ): Service {
    return dag
      .container()
      .from("postgres:16")
      .withExposedPort(5432)
      .withEnvVariable("POSTGRES_DB", database)
      .withEnvVariable("POSTGRES_USER", user)
      .withEnvVariable("POSTGRES_PASSWORD", password)
      .asService();
  }

  /**
   * Start Redis service
   */
  @func()
  redis(): Service {
    return dag
      .container()
      .from("redis:alpine")
      .withExposedPort(6379)
      .asService();
  }

  /**
   * Start all databases for integration testing
   */
  @func()
  async allDatabases(): Promise<Container> {
    const postgres = this.postgres();
    const redis = this.redis();
    const qdrant = this.qdrant();
    const neo4j = this.neo4j();
    const arangodb = this.arangodb();

    return dag
      .container()
      .from("alpine:latest")
      .withServiceBinding("postgres", postgres)
      .withServiceBinding("redis", redis)
      .withServiceBinding("qdrant", qdrant)
      .withServiceBinding("neo4j", neo4j)
      .withServiceBinding("arangodb", arangodb)
      .withEnvVariable("POSTGRES_URL", "postgres://myuser:mypassword@postgres:5432/mydatabase")
      .withEnvVariable("REDIS_URL", "redis://redis:6379")
      .withEnvVariable("QDRANT_URL", "http://qdrant:6333")
      .withEnvVariable("NEO4J_URI", "bolt://neo4j:7687")
      .withEnvVariable("NEO4J_USER", "neo4j")
      .withEnvVariable("NEO4J_PASSWORD", "password123")
      .withEnvVariable("ARANGO_URL", "http://arangodb:8529")
      .withEnvVariable("ARANGO_USER", "root")
      .withEnvVariable("ARANGO_PASSWORD", "rootpassword");
  }

  // ============================================================
  // RUST BUILD & TEST
  // ============================================================

  /**
   * Get Rust build container with caching (uses nightly for edition 2024)
   */
  @func()
  rustBuilder(): Container {
    const cargoCache = dag.cacheVolume("cargo-cache");
    const targetCache = dag.cacheVolume("rust-target-cache");

    return dag
      .container()
      .from("rustlang/rust:nightly")
      .withExec(["apt-get", "update"])
      .withExec([
        "apt-get",
        "install",
        "-y",
        "pkg-config",
        "libssl-dev",
        "protobuf-compiler",
      ])
      // Disable sccache wrapper from .cargo/config.toml (not installed in container)
      .withEnvVariable("RUSTC_WRAPPER", "")
      .withMountedCache("/usr/local/cargo/registry", cargoCache)
      .withMountedCache("/app/target", targetCache)
      .withWorkdir("/app");
  }

  /**
   * Build all Rust packages
   */
  @func()
  async build(source: Directory): Promise<Container> {
    return this.rustBuilder()
      .withDirectory("/app", source)
      .withExec(["cargo", "build", "--release"]);
  }

  /**
   * Run cargo check
   */
  @func()
  async check(source: Directory): Promise<string> {
    return this.rustBuilder()
      .withDirectory("/app", source)
      .withExec(["cargo", "check", "--all-targets"])
      .stdout();
  }

  /**
   * Run cargo clippy
   */
  @func()
  async lint(source: Directory): Promise<string> {
    return this.rustBuilder()
      .withDirectory("/app", source)
      .withExec(["rustup", "component", "add", "clippy"])
      .withExec(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"])
      .stdout();
  }

  /**
   * Run cargo fmt check
   */
  @func()
  async formatCheck(source: Directory): Promise<string> {
    return this.rustBuilder()
      .withDirectory("/app", source)
      .withExec(["rustup", "component", "add", "rustfmt"])
      .withExec(["cargo", "fmt", "--all", "--check"])
      .stdout();
  }

  /**
   * Run tests
   */
  @func()
  async test(source: Directory): Promise<string> {
    return this.rustBuilder()
      .withDirectory("/app", source)
      .withExec(["cargo", "test", "--all"])
      .stdout();
  }

  /**
   * Run tests with database services
   */
  @func()
  async testWithDatabases(source: Directory): Promise<string> {
    const postgres = this.postgres();
    const redis = this.redis();

    return this.rustBuilder()
      .withDirectory("/app", source)
      .withServiceBinding("postgres", postgres)
      .withServiceBinding("redis", redis)
      .withEnvVariable(
        "DATABASE_URL",
        "postgres://myuser:mypassword@postgres:5432/mydatabase"
      )
      .withEnvVariable("REDIS_URL", "redis://redis:6379")
      .withExec(["cargo", "test", "--all"])
      .stdout();
  }

  // ============================================================
  // CONTAINER BUILDS
  // ============================================================

  /**
   * Build zerg-api container image
   */
  @func()
  async buildZergApi(source: Directory): Promise<Container> {
    const builder = await this.build(source);

    return dag
      .container()
      .from("debian:bookworm-slim")
      .withExec(["apt-get", "update"])
      .withExec(["apt-get", "install", "-y", "ca-certificates"])
      .withExec(["rm", "-rf", "/var/lib/apt/lists/*"])
      .withFile(
        "/usr/local/bin/zerg-api",
        builder.file("/app/target/release/zerg_api")
      )
      .withExposedPort(8080)
      .withEntrypoint(["/usr/local/bin/zerg-api"]);
  }

  /**
   * Build zerg-tasks container image
   */
  @func()
  async buildZergTasks(source: Directory): Promise<Container> {
    const builder = await this.build(source);

    return dag
      .container()
      .from("debian:bookworm-slim")
      .withExec(["apt-get", "update"])
      .withExec(["apt-get", "install", "-y", "ca-certificates"])
      .withExec(["rm", "-rf", "/var/lib/apt/lists/*"])
      .withFile(
        "/usr/local/bin/zerg-tasks",
        builder.file("/app/target/release/zerg_tasks")
      )
      .withExposedPort(50051)
      .withEntrypoint(["/usr/local/bin/zerg-tasks"]);
  }

  /**
   * Publish container to the registry
   */
  @func()
  async publish(
    container: Container,
    registry: string,
    repository: string,
    tag: string = "latest"
  ): Promise<string> {
    const address = `${registry}/${repository}:${tag}`;
    return container.publish(address);
  }

  // ============================================================
  // CI PIPELINE
  // ============================================================

  /**
   * Run full CI pipeline (check, lint, format, test)
   */
  @func()
  async ci(source: Directory): Promise<string> {
    const results: string[] = [];

    // Run checks in parallel
    const [checkResult, lintResult, fmtResult, testResult] = await Promise.all([
      this.check(source).catch((e) => `Check failed: ${e.message}`),
      this.lint(source).catch((e) => `Lint failed: ${e.message}`),
      this.formatCheck(source).catch((e) => `Format check failed: ${e.message}`),
      this.test(source).catch((e) => `Test failed: ${e.message}`),
    ]);

    results.push("=== Check ===", checkResult);
    results.push("=== Lint ===", lintResult);
    results.push("=== Format ===", fmtResult);
    results.push("=== Test ===", testResult);

    return results.join("\n");
  }

  /**
   * Run full CI/CD pipeline (CI + build containers + publish)
   */
  @func()
  async cicd(
    source: Directory,
    registry: string = "ghcr.io",
    repository: string = "yurikrupnik/nx-playground",
    tag: string = "latest",
    push: boolean = false
  ): Promise<string> {
    const results: string[] = [];

    // Run CI
    const ciResult = await this.ci(source);
    results.push(ciResult);

    // Build containers
    results.push("\n=== Building Containers ===");
    const [apiContainer, tasksContainer] = await Promise.all([
      this.buildZergApi(source),
      this.buildZergTasks(source),
    ]);
    results.push("Built zerg-api container");
    results.push("Built zerg-tasks container");

    // Publish if requested
    if (push) {
      results.push("\n=== Publishing Containers ===");
      const [apiAddr, tasksAddr] = await Promise.all([
        this.publish(apiContainer, registry, `${repository}/zerg-api`, tag),
        this.publish(tasksContainer, registry, `${repository}/zerg-tasks`, tag),
      ]);
      results.push(`Published: ${apiAddr}`);
      results.push(`Published: ${tasksAddr}`);
    }

    return results.join("\n");
  }

  // ============================================================
  // DEVELOPMENT HELPERS
  // ============================================================

  /**
   * Start development environment with all databases
   */
  @func()
  async devEnv(source: Directory): Promise<Container> {
    const dbContainer = await this.allDatabases();

    return dbContainer
      .withDirectory("/app", source)
      .withWorkdir("/app")
      .withExec(["sh", "-c", "echo 'Development environment ready. Databases available at:'"])
      .withExec(["sh", "-c", "echo '  PostgreSQL: postgres:5432'"])
      .withExec(["sh", "-c", "echo '  Redis: redis:6379'"])
      .withExec(["sh", "-c", "echo '  Qdrant: qdrant:6333'"])
      .withExec(["sh", "-c", "echo '  Neo4j: neo4j:7687'"])
      .withExec(["sh", "-c", "echo '  ArangoDB: arangodb:8529'"]);
  }
}
