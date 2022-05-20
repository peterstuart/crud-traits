use crate::Meta;

pub trait IntoId<Id: Send + Sync> {
    fn into_id(&self) -> Id;
}

impl<T: Meta> IntoId<T::Id> for T {
    fn into_id(&self) -> T::Id {
        self.id()
    }
}

impl IntoId<i32> for i32 {
    fn into_id(&self) -> i32 {
        *self
    }
}
