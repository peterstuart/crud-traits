use crate::Read;
use async_trait::async_trait;

/// A parent/child relationship where the parent may have multiple
/// children.
#[async_trait]
pub trait HasMany<Child>
where
    Child: Read,
    Child::Id: Send + Sync,
{
    async fn child_ids(&self) -> Result<Vec<<Child as Read>::Id>, Child::Error>;

    async fn children(
        &self,
        store: &<Child as Read>::Store,
    ) -> Result<Vec<Child>, <Child as Read>::Error> {
        let ids = self.child_ids().await?;
        Child::read_many(&ids, store).await
    }
}
