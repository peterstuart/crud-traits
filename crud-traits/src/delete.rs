use async_trait::async_trait;

use crate::Meta;

/// Delete record with a given ID.
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
///     async fn delete(id: i32, store: &PgPool) -> Result<(), Error>
///     {
///         sqlx::query("DELETE FROM users WHERE id = $1")
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
    Self: Meta + Sized,
{
    async fn delete(id: Self::Id, store: &Self::Store) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait DeleteSelf
where
    Self: Meta + Delete,
{
    async fn delete(self, store: &Self::Store) -> Result<(), Self::Error> {
        let id = self.id();
        <Self as Delete>::delete(id, store).await
    }
}

impl<T> DeleteSelf for T
where
    T: Delete,
    <T as Meta>::Id: Send,
{
}
