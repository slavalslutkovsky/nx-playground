pub use sea_orm_migration::prelude::*;

mod m20241128_000000_bootstrap;
mod m20241129_000000_create_users;
mod m20241129_000001_create_projects;
mod m20241129_000002_create_cloud_resources;
mod m20241201_000000_seed_initial_data;
mod m20241206_000000_create_tasks;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241128_000000_bootstrap::Migration),
            Box::new(m20241129_000000_create_users::Migration),
            Box::new(m20241129_000001_create_projects::Migration),
            Box::new(m20241129_000002_create_cloud_resources::Migration),
            Box::new(m20241201_000000_seed_initial_data::Migration),
            Box::new(m20241206_000000_create_tasks::Migration),
        ]
    }
}
