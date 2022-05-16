use crate::{BelongsTo, Read};
use async_trait::async_trait;

/// A parent/child relationship where the parent may have multiple
/// children.
#[async_trait]
pub trait HasMany<Child>
where
    Self: Read + Send + Sync,
    Child: BelongsTo<Self> + Read + Send + Sync,
{
    async fn children(&self, store: &Child::Store) -> Result<Vec<Child>, Child::Error> {
        Child::for_parent_id(self.id(), store).await
    }
}
