use crate::Meta;
use async_trait::async_trait;

/// Read records with a given ID or IDs.
///
/// ## Example Implementation
///
/// ```
/// use async_trait::async_trait;
/// use crud_traits::{Meta, Read};
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
/// impl Read for User {
///     async fn read(id: i32, store: &PgPool) -> Result<Self, Error>
///     {
///         sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
///             .bind(id)
///             .fetch_one(store)
///             .await
///     }
///
///     async fn maybe_read(id: i32, store: &PgPool) -> Result<Option<Self>, Error>
///     {
///         sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
///             .bind(id)
///             .fetch_optional(store)
///             .await
///     }
///
///     async fn read_many(
///         ids: &[Self::Id],
///         store: &Self::Store,
///     ) -> Result<Vec<Self>, Self::Error> {
///         sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ANY($1)")
///             .bind(ids)
///             .fetch_all(store)
///             .await
///     }
/// }
/// ```
#[async_trait]
pub trait Read
where
    Self: Meta + Sized,
{
    /// Read a single value from the database, returning an error if
    /// it is not present.
    async fn read(id: Self::Id, store: &Self::Store) -> Result<Self, Self::Error>;

    /// Reads an optional value from the database.
    async fn maybe_read(id: Self::Id, store: &Self::Store) -> Result<Option<Self>, Self::Error>;

    /// Reads many values from the database.
    async fn read_many(ids: &[Self::Id], store: &Self::Store) -> Result<Vec<Self>, Self::Error>;

    /// Update `self` by reloading its data using [`read`](Read::read).
    async fn reload(&mut self, store: &Self::Store) -> Result<(), Self::Error> {
        let id = self.id();
        *self = Self::read(id, store).await?;

        Ok(())
    }
}
