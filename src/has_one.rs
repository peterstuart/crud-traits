use crate::Read;
use async_trait::async_trait;

/// A parent/child relationship where the parent has one child.
#[async_trait]
pub trait HasOne<Child>
where
    Child: Read,
    Child::Id: Send + Sync,
{
    async fn child_id(&self) -> Result<<Child as Read>::Id, Child::Error>;

    async fn child(&self, store: &<Child as Read>::Store) -> Result<Child, <Child as Read>::Error> {
        let id = self.child_id().await?;
        Child::read(id, store).await
    }
}
