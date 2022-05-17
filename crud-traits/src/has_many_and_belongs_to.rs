use crate::{Meta, Read};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait HasManyAndBelongsTo<Parent>
where
    Self: Clone + Read,
    Parent: Meta<Store = Self::Store, Error = Self::Error> + Read,
{
    async fn parent_ids(&self, store: &Self::Store) -> Result<Vec<Parent::Id>, Self::Error>;

    async fn parents(&self, store: &Parent::Store) -> Result<Vec<Parent>, Self::Error> {
        let ids = self.parent_ids(store).await?;
        Parent::read_many(&ids, store).await
    }

    async fn for_parent_id(
        id: Parent::Id,
        store: &Parent::Store,
    ) -> Result<Vec<Self>, Self::Error> {
        let ids = vec![id.clone()];
        let values = Self::for_parent_ids(&ids, store).await?;
        Ok(values.get(&id).cloned().unwrap_or_else(|| vec![]))
    }

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Parent::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error>;
}
