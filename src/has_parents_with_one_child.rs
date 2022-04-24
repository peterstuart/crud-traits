use crate::{HasOne, Read};
use async_trait::async_trait;
use std::{collections::HashSet, hash::Hash};

/// The inverse of [`HasOne`](crate::HasOne).
///
/// You do not need to manually implement this trait.
#[async_trait]
pub trait HasParentsWithOneChild<Parent>
where
    Self: Read,
    Self::Id: Send + Sync,
    Parent: HasOne<Self>,
{
    async fn for_parent(parent: &Parent, store: &Self::Store) -> Result<Self, Self::Error>;

    async fn for_parents(parents: &[Parent], store: &Self::Store)
        -> Result<Vec<Self>, Self::Error>;
}

#[async_trait]
impl<Parent, Child> HasParentsWithOneChild<Parent> for Child
where
    Parent: HasOne<Child> + Sync,
    Child: Read,
    Child::Id: Eq + Hash + Send + Sync,
{
    async fn for_parent(parent: &Parent, store: &Self::Store) -> Result<Self, Self::Error> {
        parent.child(store).await
    }

    async fn for_parents(
        parents: &[Parent],
        store: &Self::Store,
    ) -> Result<Vec<Self>, Self::Error> {
        let mut ids = HashSet::new();

        for parent in parents {
            let id = parent.child_id().await?;
            ids.insert(id);
        }

        let ids: Vec<_> = ids.into_iter().collect();

        Self::read_many(&ids, store).await
    }
}
