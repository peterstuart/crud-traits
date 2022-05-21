use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// ## Example Implementation
///
/// ```
/// use crud_traits::Meta;
/// use sqlx::{Error, PgPool};
///
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// impl Meta for User {
///     type Id = i32;
///     type Store = PgPool;
///     type Error = Error;
///
///     fn id(&self) -> Self::Id {
///         self.id
///     }
/// }
/// ```
pub trait Meta {
    type Id: 'static + Clone + Debug + Eq + Hash + Send + Sync;
    type Store: Send + Sync;

    /// The error type used in all `Result`s returned by functions in
    /// CRUD trait implementations for this type.
    type Error;

    /// A unique ID for the record.
    ///
    /// Typically the primary key for the record in a database.
    fn id(&self) -> Self::Id;
}

/// Produces a hash map of IDs to values given some values which
/// implement [`Meta`](crate::Meta).
pub fn hash_map_by_id<T>(values: Vec<T>) -> HashMap<T::Id, T>
where
    T: Meta,
    T::Id: Eq + Hash,
{
    values
        .into_iter()
        .map(|value| (value.id(), value))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::{Error, PgPool};

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Person {
        id: i32,
        name: String,
    }

    impl Meta for Person {
        type Id = i32;
        type Store = PgPool;
        type Error = Error;

        fn id(&self) -> Self::Id {
            self.id
        }
    }

    #[test]
    fn hash_by_id() {
        let person1 = Person {
            id: 1,
            name: "Person 1".into(),
        };
        let person2 = Person {
            id: 2,
            name: "Person 2".into(),
        };

        let people = vec![person1.clone(), person2.clone()];

        let hash = super::hash_map_by_id(people);

        assert_eq!(hash.get(&1), Some(&person1));
        assert_eq!(hash.get(&2), Some(&person2));
        assert_eq!(hash.get(&3), None);
    }
}
