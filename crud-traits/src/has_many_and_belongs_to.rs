use crate::{Meta, Read};
use async_trait::async_trait;

#[async_trait]
pub trait HasManyAndBelongsTo<Parent>
where
    Self: Read,
    Parent: Meta<Error = Self::Error> + Read,
{
    async fn parent_ids(&self) -> Result<Vec<Parent::Id>, Self::Error>;

    async fn for_parent(id: Parent::Id, store: &Parent::Store) -> Result<Vec<Self>, Self::Error> {
        let ids = vec![id];
        Self::for_parents(&ids, store).await
    }

    async fn for_parents(
        ids: &[Parent::Id],
        store: &Parent::Store,
    ) -> Result<Vec<Self>, Self::Error>;

    async fn parents(&self, store: &Parent::Store) -> Result<Vec<Parent>, Self::Error> {
        let ids = self.parent_ids().await?;
        Parent::read_many(&ids, store).await
    }
}
