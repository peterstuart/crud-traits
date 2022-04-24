use super::Id;
use async_trait::async_trait;

/// Update a record.
///
/// ## Example Implementation
///
/// ```
/// use async_trait::async_trait;
/// use crud_traits::Update;
/// use sqlx::{Error, FromRow, PgPool};
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     first_name: String,
///     last_name: String,
/// }
///
/// struct Input {
///     first_name: String,
///     last_name: String,
/// }
///
/// #[async_trait]
/// impl Update for User {
///     type Id = i32;
///     type Input = Input;
///     type Store = PgPool;
///     type Error = Error;
///
///     async fn update(id: i32, input: Input, store: &PgPool) -> Result<Self, Error>
///     {
///         sqlx::query_as::<_, User>(
///             "UPDATE users SET first_name = ?, last_name = ? WHERE id = ? RETURNING *",
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
    Self: Sized,
{
    type Id;
    type Input;
    type Store: Send + Sync;
    type Error;

    async fn update(
        id: Self::Id,
        input: Self::Input,
        store: &Self::Store,
    ) -> Result<Self, Self::Error>;
}

#[async_trait]
pub trait UpdateSelf
where
    Self: Id + Update<Id = <Self as Id>::Id>,
    <Self as Id>::Id: Send,
    Self::Input: Send + Sync,
{
    async fn update(&mut self, input: Self::Input, store: &Self::Store) -> Result<(), Self::Error> {
        let id = self.id();

        *self = <Self as Update>::update(id, input, store).await?;

        Ok(())
    }
}

impl<T> UpdateSelf for T
where
    T: Id + Update<Id = <T as Id>::Id>,
    <T as Id>::Id: Send,
    <T as Update>::Input: Send + Sync,
{
}
