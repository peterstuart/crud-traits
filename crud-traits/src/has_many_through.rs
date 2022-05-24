use crate::{as_ids, hash_map_by_id, AsId, Meta, Read};
use async_trait::async_trait;
use std::{collections::HashMap, slice};

/// Represents a many-to-many relationship between two types.
#[async_trait]
pub trait HasManyThrough<Relation>
where
    Self: Clone + Meta + Read + Send + Sync,
    Relation: Meta<Store = Self::Store, Error = Self::Error> + Read,
{
    async fn relation_ids(&self, store: &Self::Store) -> Result<Vec<Relation::Id>, Self::Error>;

    async fn relation_ids_for_many(
        ids: &[Self::Id],
        store: &Self::Store,
    ) -> Result<HashMap<Self::Id, Vec<Relation::Id>>, Self::Error>;

    async fn ids_for_relation_ids(
        ids: &[Relation::Id],
        store: &Relation::Store,
    ) -> Result<HashMap<Relation::Id, Vec<Self::Id>>, Self::Error>;

    async fn relations(&self, store: &Relation::Store) -> Result<Vec<Relation>, Self::Error> {
        let ids = self.relation_ids(store).await?;
        Relation::read_many(&ids, store).await
    }

    async fn set_relations<T>(relations: &[T]) -> Result<(), Self::Error>
    where
        T: AsId<Id = Relation::Id> + Send + Sync;

    async fn ids_for_relations<T>(
        relations: &[T],
        store: &Relation::Store,
    ) -> Result<HashMap<Relation::Id, Vec<Self::Id>>, Self::Error>
    where
        T: AsId<Id = Relation::Id> + Send + Sync,
    {
        let ids = as_ids(relations);
        Self::ids_for_relation_ids(&ids, store).await
    }

    async fn ids_for_relation<T>(
        relation: T,
        store: &Relation::Store,
    ) -> Result<Vec<Self::Id>, Self::Error>
    where
        T: AsId<Id = Relation::Id> + Send + Sync,
    {
        let id = relation.as_id();
        Ok(Self::ids_for_relation_ids(&[id.clone()], store)
            .await?
            .remove(&id)
            .unwrap_or_default())
    }

    async fn for_relations<T>(
        relations: &[T],
        store: &Relation::Store,
    ) -> Result<HashMap<Relation::Id, Vec<Self>>, Self::Error>
    where
        T: AsId<Id = Relation::Id> + Send + Sync,
    {
        let ids = as_ids(relations);
        let ids_by_relation_id = Self::ids_for_relation_ids(&ids, store).await?;
        let all_ids: Vec<_> = ids_by_relation_id.values().flatten().cloned().collect();
        let all_values = Self::read_many(&all_ids, store).await?;
        let children_by_id = hash_map_by_id(all_values);

        Ok(ids_by_relation_id
            .into_iter()
            .map(|(relation_id, child_ids)| {
                let children = child_ids
                    .into_iter()
                    .filter_map(|child_id| children_by_id.get(&child_id).cloned())
                    .collect();
                (relation_id, children)
            })
            .collect())
    }

    async fn for_relation<T>(
        relation: &T,
        store: &Relation::Store,
    ) -> Result<Vec<Self>, Self::Error>
    where
        T: AsId<Id = Relation::Id> + Send + Sync,
    {
        let id = relation.as_id();
        Ok(Self::for_relations(slice::from_ref(relation), store)
            .await?
            .remove(&id)
            .unwrap_or_default())
    }
}
