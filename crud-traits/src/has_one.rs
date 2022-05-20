use crate::{BelongsTo, Meta, Read};
use async_trait::async_trait;

/// Represents a one-to-one relationship between two types.
///
/// `HasOne` is the inverse of [`BelongsTo`](crate::BelongsTo).
///
/// `Child` refers to the `BelongsTo` type in the relationship.
#[async_trait]
pub trait HasOne<Child>
where
    Self: Clone + Meta + Read + Send + Sync,
    Child: BelongsTo<Self> + Send,
{
    async fn child(&self, store: &Child::Store) -> Result<Child, Child::Error> {
        Ok(Child::for_parent(self, store).await?.remove(0))
    }
}
