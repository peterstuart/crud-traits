use crate::{HasMany, Read};
use async_trait::async_trait;
use std::{collections::HashSet, hash::Hash};

/// The inverse of [`HasMany`](crate::HasMany).
///
/// You do not need to manually implement this trait.
#[async_trait]
pub trait HasParentsWithManyChildren<Parent>
where
    Self: Read,
    Self::Id: Send + Sync,
    Parent: HasMany<Self>,
{
    async fn for_parent(parent: &Parent, store: &Self::Store) -> Result<Vec<Self>, Self::Error>;

    async fn for_parents(parents: &[Parent], store: &Self::Store)
        -> Result<Vec<Self>, Self::Error>;
}

#[async_trait]
impl<Parent, Child> HasParentsWithManyChildren<Parent> for Child
where
    Parent: HasMany<Child> + Sync,
    Child: Read,
    Child::Id: Eq + Hash + Send + Sync,
{
    async fn for_parent(parent: &Parent, store: &Self::Store) -> Result<Vec<Self>, Self::Error> {
        parent.children(store).await
    }

    async fn for_parents(
        parents: &[Parent],
        store: &Self::Store,
    ) -> Result<Vec<Self>, Self::Error> {
        let mut ids = HashSet::new();

        for parent in parents {
            let parent_ids = parent.child_ids().await?;
            for id in parent_ids {
                ids.insert(id);
            }
        }

        let ids: Vec<_> = ids.into_iter().collect();

        Self::read_many(&ids, store).await
    }
}
