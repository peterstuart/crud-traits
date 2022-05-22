use crate::Meta;

pub trait AsId {
    type Id: Send + Sync;

    fn as_id(&self) -> Self::Id;
}

impl<T> AsId for T
where
    T: Meta,
{
    type Id = <T as Meta>::Id;

    fn as_id(&self) -> Self::Id {
        self.id()
    }
}

impl AsId for i32 {
    type Id = Self;

    fn as_id(&self) -> i32 {
        *self
    }
}

pub fn as_ids<T: AsId>(values: &[T]) -> Vec<T::Id> {
    values.iter().map(AsId::as_id).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    struct Model {
        id: i32,
    }

    impl AsId for Model {
        type Id = i32;

        fn as_id(&self) -> i32 {
            self.id
        }
    }

    #[test]
    fn as_ids() {
        let models = vec![Model { id: 1 }, Model { id: 2 }, Model { id: 3 }];
        let ids = super::as_ids(&models);

        assert_eq!(ids, vec![1, 2, 3]);
    }
}
