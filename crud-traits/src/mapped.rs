use crate::{hash_map_by_id, BelongsTo, Create, Delete, Meta, Read, Update};
use async_trait::async_trait;
use std::collections::HashMap;

/// Allows a type to implement the [`Create`](crate::Create],
/// [`Read`](crate::Read), [`Update`](crate::Update),
/// [`Delete`](crate::Delete), and [`BelongsTo`](crate::BelongsTo)
/// traits by delegating to an underlying type (`OriginalModel`).
#[async_trait]
pub trait Mapped: Clone + Send + Sized + Sync {
    type OriginalModel: Meta + Send + Sync;
    type Error: From<<Self::OriginalModel as Meta>::Error>;

    fn id(&self) -> <Self::OriginalModel as Meta>::Id;

    async fn from(
        value: Self::OriginalModel,
        store: &<Self::OriginalModel as Meta>::Store,
    ) -> Result<Self, Self::Error>;

    async fn from_many(
        values: Vec<Self::OriginalModel>,
        store: &<Self::OriginalModel as Meta>::Store,
    ) -> Result<Vec<Self>, Self::Error>;
}

#[async_trait]
impl<MappedModel> Meta for MappedModel
where
    MappedModel: Mapped,
{
    type Id = <MappedModel::OriginalModel as Meta>::Id;
    type Store = <MappedModel::OriginalModel as Meta>::Store;
    type Error = MappedModel::Error;

    fn id(&self) -> Self::Id {
        MappedModel::id(self)
    }
}

#[async_trait]
impl<MappedModel> Create for MappedModel
where
    MappedModel: Mapped,
    MappedModel::OriginalModel: Create,
{
    type Input = <MappedModel::OriginalModel as Create>::Input;

    async fn create(input: Self::Input, store: &Self::Store) -> Result<Self, Self::Error> {
        let original = MappedModel::OriginalModel::create(input, store).await?;
        Self::from(original, store).await
    }
}

#[async_trait]
impl<MappedModel> Read for MappedModel
where
    MappedModel: Mapped,
    MappedModel::OriginalModel: Read,
{
    async fn read(id: Self::Id, store: &Self::Store) -> Result<Self, Self::Error> {
        let original = MappedModel::OriginalModel::read(id, store).await?;
        Self::from(original, store).await
    }

    async fn read_optional(id: Self::Id, store: &Self::Store) -> Result<Option<Self>, Self::Error> {
        let original = MappedModel::OriginalModel::read_optional(id, store).await?;
        Ok(match original {
            Some(original) => Some(Self::from(original, store).await?),
            None => None,
        })
    }

    async fn read_many(ids: &[Self::Id], store: &Self::Store) -> Result<Vec<Self>, Self::Error> {
        let originals = MappedModel::OriginalModel::read_many(ids, store).await?;
        Self::from_many(originals, store).await
    }
}

#[async_trait]
impl<MappedModel> Update for MappedModel
where
    MappedModel: Mapped,
    MappedModel::OriginalModel: Update,
{
    type Input = <MappedModel::OriginalModel as Update>::Input;

    async fn update(
        id: Self::Id,
        input: Self::Input,
        store: &Self::Store,
    ) -> Result<Self, Self::Error> {
        let original = MappedModel::OriginalModel::update(id, input, store).await?;
        Self::from(original, store).await
    }
}

#[async_trait]
impl<MappedModel> Delete for MappedModel
where
    MappedModel: Mapped,
    MappedModel::OriginalModel: Delete,
{
    async fn delete(id: Self::Id, store: &Self::Store) -> Result<(), Self::Error> {
        Ok(MappedModel::OriginalModel::delete(id, store).await?)
    }
}

pub trait MappedWithParentId<Parent>
where
    Parent: Meta,
{
    fn parent_id(&self) -> <Parent as Meta>::Id;
}

#[async_trait]
impl<MappedModel, Parent> BelongsTo<Parent> for MappedModel
where
    MappedModel: Mapped + MappedWithParentId<Parent>,
    <MappedModel as Mapped>::OriginalModel: BelongsTo<Parent> + Clone,
    Parent: Clone + Meta + Read + Send + Sync,
{
    fn parent_id(&self) -> <Parent as Meta>::Id {
        MappedWithParentId::parent_id(self)
    }

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Self::Store,
    ) -> Result<HashMap<Parent::Id, Vec<Self>>, Self::Error> {
        let hash_map = <MappedModel as Mapped>::OriginalModel::for_parent_ids(ids, store).await?;
        let values: Vec<<MappedModel as Mapped>::OriginalModel> =
            hash_map.values().flatten().cloned().collect();
        let values_by_id = hash_map_by_id(Self::from_many(values, store).await?);

        Ok(hash_map
            .into_iter()
            .map(|(parent_id, children)| {
                (
                    parent_id,
                    children
                        .iter()
                        .flat_map(|child| values_by_id.get(&child.id()).cloned())
                        .collect(),
                )
            })
            .collect())
    }
}
