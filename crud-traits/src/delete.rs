use async_trait::async_trait;

use crate::Meta;

/// Delete records
///
/// ## Example Implementation
///
/// ```
/// use async_trait::async_trait;
/// use crud_traits::{Delete, Meta};
/// use sqlx::{Error, FromRow, PgPool};
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     first_name: String,
///     last_name: String,
/// }
///
/// impl Meta for User {
///     type Id = i32;
///     type Store = PgPool;
///     type Error = Error;
///
///     fn id(&self) -> i32 {
///         self.id
///     }
/// }
///
/// #[async_trait]
/// impl Delete for User {
///     async fn delete_by_id(id: i32, store: &PgPool) -> Result<(), Error>
///     {
///         sqlx::query("DELETE FROM users WHERE id = $1")
///             .bind(id)
///             .execute(store)
///             .await?;
///         Ok(())
///     }
///
///     async fn delete_all(store: &PgPool) -> Result<(), Error>
///     {
///         sqlx::query("DELETE FROM users")
///             .execute(store)
///             .await?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Delete
where
    Self: Meta + Sized,
{
    async fn delete_by_id(id: Self::Id, store: &Self::Store) -> Result<(), Self::Error>;

    async fn delete(self, store: &Self::Store) -> Result<(), Self::Error> {
        let id = self.id();
        <Self as Delete>::delete_by_id(id, store).await
    }

    async fn delete_all(store: &Self::Store) -> Result<(), Self::Error>;
}
