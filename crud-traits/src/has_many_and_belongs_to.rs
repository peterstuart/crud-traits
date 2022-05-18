use crate::{hash_map_by_id, Meta, Read};
use async_trait::async_trait;
use std::collections::HashMap;

/// Represents a many-to-many relationship between two types, usually
/// where the relationship is stored in a join table.
///
/// `Parent` refers to the other (not `Self`) type in the
/// relationship.
#[async_trait]
pub trait HasManyAndBelongsTo<Parent>
where
    Self: Clone + Read,
    Parent: Meta<Store = Self::Store, Error = Self::Error> + Read,
{
    async fn parent_ids(&self, store: &Self::Store) -> Result<Vec<Parent::Id>, Self::Error>;

    async fn parent_ids_for_many(
        ids: &[Self::Id],
        store: &Self::Store,
    ) -> Result<HashMap<Self::Id, Vec<Parent::Id>>, Self::Error>;

    async fn ids_for_parent_ids(
        ids: &[Parent::Id],
        store: &Parent::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self::Id>>, Self::Error>;

    async fn ids_for_parent_id(
        id: Parent::Id,
        store: &Parent::Store,
    ) -> Result<Vec<Self::Id>, Self::Error> {
        let ids = vec![id.clone()];
        let values = Self::ids_for_parent_ids(&ids, store).await?;
        Ok(values.get(&id).cloned().unwrap_or_default())
    }

    async fn parents(&self, store: &Parent::Store) -> Result<Vec<Parent>, Self::Error> {
        let ids = self.parent_ids(store).await?;
        Parent::read_many(&ids, store).await
    }

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Parent::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error> {
        let ids_by_parent_id = Self::ids_for_parent_ids(ids, store).await?;
        let all_ids: Vec<_> = ids_by_parent_id
            .iter()
            .flat_map(|(_, ids)| ids)
            .cloned()
            .collect();
        let all_values = Self::read_many(&all_ids, store).await?;
        let children_by_id = hash_map_by_id(all_values);

        Ok(ids_by_parent_id
            .into_iter()
            .map(|(parent_id, child_ids)| {
                let children = child_ids
                    .into_iter()
                    .flat_map(|child_id| children_by_id.get(&child_id).cloned())
                    .collect();
                (parent_id, children)
            })
            .collect())
    }
}
