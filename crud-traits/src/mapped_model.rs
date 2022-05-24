use crate::{hash_map_by_id, BelongsTo, Create, Delete, Meta, Read, Update};
use async_trait::async_trait;
use std::collections::HashMap;

/// Allows a type to implement the [`Create`](crate::Create],
/// [`Read`](crate::Read), [`Update`](crate::Update),
/// [`Delete`](crate::Delete), and [`BelongsTo`](crate::BelongsTo)
/// traits by delegating to an underlying type (`OriginalModel`).
#[async_trait]
pub trait MappedModel: Clone + Send + Sized + Sync {
    type OriginalModel: Meta + Send + Sync;
    type Error: From<<Self::OriginalModel as Meta>::Error>;

    fn id(&self) -> <Self::OriginalModel as Meta>::Id;

    async fn from_model(
        value: Self::OriginalModel,
        store: &<Self::OriginalModel as Meta>::Store,
    ) -> Result<Self, Self::Error>;

    async fn from_models(
        values: Vec<Self::OriginalModel>,
        store: &<Self::OriginalModel as Meta>::Store,
    ) -> Result<Vec<Self>, Self::Error>;
}

#[async_trait]
impl<T> Meta for T
where
    T: MappedModel,
{
    type Id = <T::OriginalModel as Meta>::Id;
    type Store = <T::OriginalModel as Meta>::Store;
    type Error = T::Error;

    fn id(&self) -> Self::Id {
        T::id(self)
    }
}

#[async_trait]
impl<T> Create for T
where
    T: MappedModel,
    T::OriginalModel: Create,
{
    type Input = <T::OriginalModel as Create>::Input;

    async fn create(input: Self::Input, store: &Self::Store) -> Result<Self, Self::Error> {
        let original = T::OriginalModel::create(input, store).await?;
        Self::from_model(original, store).await
    }
}

#[async_trait]
impl<T> Read for T
where
    T: MappedModel,
    T::OriginalModel: Read,
{
    async fn read(id: Self::Id, store: &Self::Store) -> Result<Self, Self::Error> {
        let original = T::OriginalModel::read(id, store).await?;
        Self::from_model(original, store).await
    }

    async fn maybe_read(id: Self::Id, store: &Self::Store) -> Result<Option<Self>, Self::Error> {
        let original = T::OriginalModel::maybe_read(id, store).await?;
        Ok(match original {
            Some(original) => Some(Self::from_model(original, store).await?),
            None => None,
        })
    }

    async fn read_many(ids: &[Self::Id], store: &Self::Store) -> Result<Vec<Self>, Self::Error> {
        let originals = T::OriginalModel::read_many(ids, store).await?;
        Self::from_models(originals, store).await
    }

    async fn read_all(store: &Self::Store) -> Result<Vec<Self>, Self::Error> {
        let originals = T::OriginalModel::read_all(store).await?;
        Self::from_models(originals, store).await
    }
}

#[async_trait]
impl<T> Update for T
where
    T: MappedModel,
    T::OriginalModel: Update,
{
    type Input = <T::OriginalModel as Update>::Input;

    async fn update_by_id(
        id: Self::Id,
        input: Self::Input,
        store: &Self::Store,
    ) -> Result<Self, Self::Error> {
        let original = T::OriginalModel::update_by_id(id, input, store).await?;
        Self::from_model(original, store).await
    }
}

#[async_trait]
impl<T> Delete for T
where
    T: MappedModel,
    T::OriginalModel: Delete,
{
    async fn delete_by_id(id: Self::Id, store: &Self::Store) -> Result<(), Self::Error> {
        Ok(T::OriginalModel::delete_by_id(id, store).await?)
    }
}

pub trait MappedModelWithParentId<Parent>
where
    Parent: Meta,
{
    fn parent_id(&self) -> <Parent as Meta>::Id;
}

#[async_trait]
impl<T, Parent> BelongsTo<Parent> for T
where
    T: MappedModel + MappedModelWithParentId<Parent>,
    <T as MappedModel>::OriginalModel: BelongsTo<Parent> + Clone,
    Parent: Clone + Meta + Read + Send + Sync,
{
    fn parent_id(&self) -> <Parent as Meta>::Id {
        MappedModelWithParentId::parent_id(self)
    }

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Self::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error> {
        let hash_map = <T as MappedModel>::OriginalModel::for_parent_ids(ids, store).await?;
        let values: Vec<<T as MappedModel>::OriginalModel> =
            hash_map.values().flatten().cloned().collect();
        let values_by_id = hash_map_by_id(Self::from_models(values, store).await?);

        Ok(hash_map
            .into_iter()
            .map(|(parent_id, children)| {
                (
                    parent_id,
                    children
                        .iter()
                        .filter_map(|child| values_by_id.get(&child.id()).cloned())
                        .collect(),
                )
            })
            .collect())
    }
}
