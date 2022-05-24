use async_trait::async_trait;

use crate::Meta;

/// Update a record.
///
/// ## Example Implementation
///
/// ```
/// use async_trait::async_trait;
/// use crud_traits::{Meta, Update};
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
/// impl Update for User {
///     type Input = Input;
///
///     async fn update_by_id(id: i32, input: Input, store: &PgPool) -> Result<Self, Error>
///     {
///         sqlx::query_as::<_, User>(
///             "UPDATE users SET first_name = $1, last_name = $2 WHERE id = $3 RETURNING *",
///         )
///         .bind(input.first_name)
///         .bind(input.last_name)
///         .bind(id)
///         .fetch_one(store)
///         .await
///     }
/// }
/// ```
#[async_trait]
pub trait Update
where
    Self: Meta + Sized,
{
    type Input: Send;

    async fn update_by_id(
        id: Self::Id,
        input: Self::Input,
        store: &Self::Store,
    ) -> Result<Self, Self::Error>;

    async fn update(&mut self, input: Self::Input, store: &Self::Store) -> Result<(), Self::Error> {
        let id = self.id();

        *self = <Self as Update>::update_by_id(id, input, store).await?;

        Ok(())
    }
}
