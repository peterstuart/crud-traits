use crate::Meta;
use async_trait::async_trait;

/// Create a record.
///
/// Typically, `Input` will not include any fields that can be
/// automatically generated, such as a database ID for the record.
///
/// ## Example Implementation
///
/// ```
/// use async_trait::async_trait;
/// use crud_traits::{Create, Meta};
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
/// struct Input {
///     first_name: String,
///     last_name: String,
/// }
///
/// #[async_trait]
/// impl Create for User {
///     type Input = Input;
///
///     async fn create(input: Input, store: &PgPool) -> Result<Self, Error>
///     {
///         sqlx::query_as::<_, User>(
///             "INSERT INTO users (first_name, last_name) VALUES ($1, $2) RETURNING *",
///         )
///         .bind(input.first_name)
///         .bind(input.last_name)
///         .fetch_one(store)
///         .await
///     }
/// }
/// ```
#[async_trait]
pub trait Create
where
    Self: Meta + Sized,
{
    type Input;

    async fn create(input: Self::Input, store: &Self::Store) -> Result<Self, Self::Error>;
}
