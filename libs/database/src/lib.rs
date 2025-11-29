use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, PrimaryKeyTrait};
use std::marker::PhantomData;
use uuid::Uuid;

/// Generic base repository for Sea-ORM entities
///
/// This provides database connection and common read operations for any entity that implements EntityTrait.
/// For create/update operations, use the entity-specific repositories.
///
/// Usage:
/// ```
/// let repo = BaseRepository::<projects::Entity>::new(db);
/// let project = repo.find_by_id(id).await?;
/// ```
pub struct BaseRepository<E>
where
    E: EntityTrait,
{
    pub db: DatabaseConnection,
    _phantom: PhantomData<E>,
}

impl<E> BaseRepository<E>
where
    E: EntityTrait,
{
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _phantom: PhantomData,
        }
    }

    /// Find entity by primary key (supports UUID)
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<E::Model>, DbErr>
    where
        <E::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
    {
        E::find_by_id(id).one(&self.db).await
    }

    /// Find all entities with optional limit and offset
    pub async fn find_all(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<E::Model>, DbErr> {
        use sea_orm::QuerySelect;

        let mut query = E::find();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query.all(&self.db).await
    }

    /// Delete entity by primary key
    pub async fn delete_by_id(&self, id: Uuid) -> Result<u64, DbErr>
    where
        <E::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
    {
        let result = E::delete_by_id(id).exec(&self.db).await?;
        Ok(result.rows_affected)
    }

    /// Insert an active model
    pub async fn insert<A>(&self, active_model: A) -> Result<E::Model, DbErr>
    where
        A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + Send,
        E::Model: sea_orm::IntoActiveModel<A>,
    {
        active_model.insert(&self.db).await
    }

    /// Update an active model
    pub async fn update<A>(&self, active_model: A) -> Result<E::Model, DbErr>
    where
        A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + Send,
        E::Model: sea_orm::IntoActiveModel<A>,
    {
        active_model.update(&self.db).await
    }

    /// Get reference to database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

/// Helper trait for entities with UUID primary keys
pub trait UuidEntity: EntityTrait
where
    <Self::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
}

// Implement for any entity that has UUID primary key
impl<E> UuidEntity for E
where
    E: EntityTrait,
    <E::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
}
