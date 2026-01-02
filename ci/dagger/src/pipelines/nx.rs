use dagger_sdk::Query;
use eyre::Result;
use tracing::info;

use crate::utils::{config::CiConfig, node_container, nx};

/// Run NX pipeline for JavaScript/TypeScript projects
pub async fn run(
    client: &Query,
    config: &CiConfig,
    target: &str,
    projects: &str,
    extra_args: &[String],
) -> Result<()> {
    info!("=== Running NX Pipeline ===");
    info!("Target: {}, Projects: {}", target, projects);

    // Resolve which projects to run
    let project_list = resolve_projects(client, config, target, projects).await?;

    if project_list.is_empty() {
        info!("No projects to run for target '{}', skipping", target);
        return Ok(());
    }

    info!("Running NX {} on projects: {:?}", target, project_list);

    let container = node_container::create(client).await?;

    // Build the NX command
    let mut args = vec![
        "bun".to_string(),
        "nx".to_string(),
        "run-many".to_string(),
        format!("--target={}", target),
        format!("--projects={}", project_list.join(",")),
    ];

    // Add extra args if provided
    for arg in extra_args {
        args.push(arg.clone());
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = container.with_exec(args_refs).stdout().await?;

    info!("NX output:\n{}", output);
    info!("=== NX Pipeline Completed ===");

    Ok(())
}

/// Resolve project list based on input
async fn resolve_projects(
    client: &Query,
    config: &CiConfig,
    target: &str,
    projects: &str,
) -> Result<Vec<String>> {
    match projects.to_lowercase().as_str() {
        "all" => {
            info!("Running on all projects with target '{}'", target);
            // Return empty to let NX handle "all" via its own logic
            Ok(vec!["".to_string()])
        }
        "affected" => {
            info!("Detecting affected projects for target '{}'", target);
            nx::get_affected_packages(client, config, target).await
        }
        _ => {
            // Comma-separated list of specific projects
            let list: Vec<String> = projects
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            info!("Using specified projects: {:?}", list);
            Ok(list)
        }
    }
}

/// Run NX lint on affected or specified projects
#[allow(dead_code)]
pub async fn lint(client: &Query, config: &CiConfig, projects: &str) -> Result<()> {
    run(client, config, "lint", projects, &[]).await
}

/// Run NX build on affected or specified projects
#[allow(dead_code)]
pub async fn build(client: &Query, config: &CiConfig, projects: &str) -> Result<()> {
    run(client, config, "build", projects, &[]).await
}

/// Run NX test on affected or specified projects
#[allow(dead_code)]
pub async fn test(client: &Query, config: &CiConfig, projects: &str) -> Result<()> {
    run(client, config, "test", projects, &[]).await
}
