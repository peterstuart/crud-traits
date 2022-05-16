use crate::{Meta, Read};
use async_trait::async_trait;

#[async_trait]
pub trait BelongsTo<Parent>
where
    Self: Read,
    Parent: Meta + Read + Send + Sync,
    <Parent as Meta>::Id: 'static,
{
    fn parent_id(&self) -> <Parent as Meta>::Id;

    async fn for_parent_id(id: Parent::Id, store: &Self::Store) -> Result<Vec<Self>, Self::Error> {
        let ids = vec![id];
        Self::for_parent_ids(&ids, store).await
    }

    async fn for_parent(parent: &Parent, store: &Self::Store) -> Result<Vec<Self>, Self::Error> {
        let ids = vec![parent.id()];
        Self::for_parent_ids(&ids, store).await
    }

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Self::Store,
    ) -> Result<Vec<Self>, Self::Error>;

    async fn for_parents(
        parents: &[Parent],
        store: &Self::Store,
    ) -> Result<Vec<Self>, Self::Error> {
        let ids: Vec<_> = parents.iter().map(|parent| parent.id()).collect();
        Self::for_parent_ids(&ids, store).await
    }

    async fn parent(&self, store: &Parent::Store) -> Result<Parent, Parent::Error> {
        Parent::read(self.parent_id(), store).await
    }
}
