use crate::Meta;

pub trait AsId<Id: Send + Sync> {
    fn as_id(&self) -> Id;
}

impl<T: Meta> AsId<T::Id> for T {
    fn as_id(&self) -> T::Id {
        self.id()
    }
}

impl AsId<i32> for i32 {
    fn as_id(&self) -> i32 {
        *self
    }
}
