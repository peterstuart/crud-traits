use crate::{BelongsTo, Create, Delete, Meta, Read, Update};
use async_trait::async_trait;

#[async_trait]
pub trait Mapped: Sized {
    type OriginalModel: Meta + Send + Sync;
    type Error: From<<Self::OriginalModel as Meta>::Error>;

    fn original(&self) -> &Self::OriginalModel;

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
        self.original().id()
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

#[async_trait]
impl<MappedModel, Parent> BelongsTo<Parent> for MappedModel
where
    MappedModel: Mapped,
    MappedModel::OriginalModel: BelongsTo<Parent>,
    Parent: Meta + Read + Send + Sync,
{
    fn parent_id(&self) -> <Parent as Meta>::Id {
        self.original().parent_id()
    }

    async fn for_parent_ids(
        ids: &[Parent::Id],
        store: &Self::Store,
    ) -> Result<Vec<Self>, Self::Error> {
        let originals = MappedModel::OriginalModel::for_parent_ids(ids, store).await?;
        Self::from_many(originals, store).await
    }
}
