use crate::{BelongsTo, Read};
use async_trait::async_trait;

/// A parent/child relationship where the parent has one child.
#[async_trait]
pub trait HasOne<Child>
where
    Self: Read + Send + Sync,
    Child: BelongsTo<Self> + Send,
{
    async fn child(&self, store: &Child::Store) -> Result<Child, Child::Error> {
        Ok(Child::for_parent_id(self.id(), store).await?.remove(0))
    }
}
