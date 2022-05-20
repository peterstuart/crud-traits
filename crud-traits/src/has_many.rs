use crate::{BelongsTo, Meta, Read};
use async_trait::async_trait;

/// Represents a one-to-many relationship between two types.
///
/// `HasMany` is the inverse of [`BelongsTo`](crate::BelongsTo).
#[async_trait]
pub trait HasMany<Child>
where
    Self: Clone + Meta + Read + Send + Sync,
    Child: BelongsTo<Self> + Read + Send + Sync,
{
    async fn children(&self, store: &Child::Store) -> Result<Vec<Child>, Child::Error> {
        Child::for_parent(self, store).await
    }
}
