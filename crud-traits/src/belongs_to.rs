use crate::{hash_map_by_id, AsId, Meta, Read};
use async_trait::async_trait;
use std::{borrow::Borrow, collections::HashMap};

/// Represents one side of a a one-to-one or one-to-many relationship
/// between two types.
///
/// `BelongsTo` is the inverse of [`HasOne`](crate::HasOne) and
/// [`HasMany`](crate::HasMany). The type which contains the foreign
/// key of the other type should be the one which implements
/// `BelongsTo`.
///
/// `Parent` refers to the `HasMany`/`HasOne` type in the
/// relationship.
#[async_trait]
pub trait BelongsTo<Parent>
where
    Self: Read + Send + Sync,
    Parent: Clone + Meta + Read + Send + Sync,
    Parent::Id: 'static,
{
    /// The foreign key of the parent type.
    fn parent_id(&self) -> Parent::Id;

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Self::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error>;

    async fn for_parent<T>(parent: &T, store: &Self::Store) -> Result<Vec<Self>, Self::Error>
    where
        T: AsId<Id = Parent::Id> + Send + Sync,
    {
        let id = parent.as_id();
        let ids = vec![id.clone()];
        Ok(Self::for_parent_ids(&ids, store)
            .await?
            .remove(&id)
            .unwrap_or_default())
    }

    async fn for_parents<T>(
        parents: &[T],
        store: &Self::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error>
    where
        T: AsId<Id = Parent::Id> + Send + Sync,
    {
        let ids: Vec<_> = parents
            .into_iter()
            .map(|parent| parent.borrow().as_id())
            .collect();
        Self::for_parent_ids(&ids, store).await
    }

    async fn for_parents2<T, B>(
        parents: &[B],
        store: &Self::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error>
    where
        T: AsId<Id = Parent::Id> + Send + Sync,
        B: Borrow<T> + Send + Sync,
    {
        let ids: Vec<_> = parents
            .into_iter()
            .map(|parent| parent.borrow().as_id())
            .collect();
        Self::for_parent_ids(&ids, store).await
    }

    async fn parent(&self, store: &Parent::Store) -> Result<Parent, Parent::Error> {
        Parent::read(self.parent_id(), store).await
    }

    async fn parents_for_many(
        values: &[Self],
        store: &Parent::Store,
    ) -> Result<HashMap<Self::Id, Parent>, Parent::Error> {
        let parent_ids: Vec<_> = values.iter().map(|child| child.parent_id()).collect();
        let parents_by_id = hash_map_by_id(Parent::read_many(&parent_ids, store).await?);

        Ok(values
            .iter()
            .filter_map(|child| Some((child.id(), parents_by_id.get(&child.parent_id())?.clone())))
            .collect())
    }
}
