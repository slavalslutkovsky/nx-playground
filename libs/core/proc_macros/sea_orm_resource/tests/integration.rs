//! Integration tests for SeaOrmResource derive macro

use core_proc_macros::{ApiResource, SeaOrmResource};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Test basic usage
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SeaOrmResource)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[test]
fn test_generated_constants() {
    assert_eq!(Model::URL, "/projects");
    assert_eq!(Model::API_URL, "/api/projects");
    assert_eq!(Model::COLLECTION, "projects");
    assert_eq!(Model::TAG, "Projects");
}

// Test with custom attributes
mod custom {
    use super::*;

    #[derive(
        Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SeaOrmResource,
    )]
    #[sea_orm(table_name = "users")]
    #[sea_orm_resource(url = "/v2/users", tag = "User Management")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub email: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[test]
    fn test_custom_url_and_tag() {
        assert_eq!(Model::URL, "/v2/users");
        assert_eq!(Model::API_URL, "/api/v2/users");
        assert_eq!(Model::COLLECTION, "users");
        assert_eq!(Model::TAG, "User Management");
    }
}

// Test with a custom collection
mod custom_collection {
    use super::*;

    #[derive(
        Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SeaOrmResource,
    )]
    #[sea_orm(table_name = "items")]
    #[sea_orm_resource(collection = "inventory_items")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[test]
    fn test_custom_collection() {
        assert_eq!(Model::COLLECTION, "inventory_items");
        assert_eq!(Model::TAG, "Inventory Items"); // Snake case to title case
    }
}
