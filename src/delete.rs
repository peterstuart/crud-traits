use super::id::Id;
use async_trait::async_trait;

/// Delete record with a given ID.
///
/// ## Example Implementation
///
/// ```
/// use async_trait::async_trait;
/// use crud_traits::Delete;
/// use sqlx::{Error, FromRow, PgPool};
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     first_name: String,
///     last_name: String,
/// }
///
/// #[async_trait]
/// impl Delete for User {
///     type Id = i32;
///     type Store = PgPool;
///     type Error = Error;

///     async fn delete(id: i32, store: &PgPool) -> Result<(), Error>
///     {
///         sqlx::query("DELETE FROM users WHERE id = ?")
///             .bind(id)
///             .execute(store)
///             .await?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Delete
where
    Self: Sized,
{
    type Id;
    type Store: Send + Sync;
    type Error;

    async fn delete(id: Self::Id, store: &Self::Store) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait DeleteSelf
where
    Self: Id + Delete<Id = <Self as Id>::Id>,
    <Self as Id>::Id: Send,
{
    async fn delete(self, store: &Self::Store) -> Result<(), <Self as Delete>::Error> {
        let id = self.id();
        <Self as Delete>::delete(id, store).await
    }
}

impl<T> DeleteSelf for T
where
    T: Id + Delete<Id = <T as Id>::Id>,
    <T as Id>::Id: Send,
{
}
