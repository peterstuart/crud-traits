use async_trait::async_trait;
use crud_traits::{BelongsTo, Create, HasMany, Meta, Read};
use crud_traits_macros::{belongs_to, has_many};
use sqlx::{Error, FromRow, PgPool};
use std::env;

#[derive(Clone, Debug, Eq, FromRow, PartialEq)]
struct Person {
    id: i32,
    name: String,
}

impl Meta for Person {
    type Id = i32;
    type Store = PgPool;
    type Error = Error;

    fn id(&self) -> i32 {
        self.id
    }
}

struct PersonInput {
    name: String,
}

#[async_trait]
impl Create for Person {
    type Input = PersonInput;

    async fn create(input: PersonInput, store: &PgPool) -> Result<Self, Error> {
        sqlx::query_as::<_, Person>("INSERT INTO people (name) VALUES ($1) RETURNING *")
            .bind(input.name)
            .fetch_one(store)
            .await
    }
}

#[async_trait]
impl Read for Person {
    async fn read(id: i32, store: &PgPool) -> Result<Self, Error> {
        sqlx::query_as::<_, Person>("SELECT * FROM people WHERE id = $1")
            .bind(id)
            .fetch_one(store)
            .await
    }

    async fn read_optional(id: i32, store: &Self::Store) -> Result<Option<Self>, Error> {
        sqlx::query_as::<_, Person>("SELECT * FROM people WHERE id = $1")
            .bind(id)
            .fetch_optional(store)
            .await
    }

    async fn read_many(ids: &[i32], store: &PgPool) -> Result<Vec<Self>, Error> {
        sqlx::query_as::<_, Person>("SELECT * FROM people WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(store)
            .await
    }
}

has_many!(Person, Dog);

#[derive(Clone, Debug, Eq, FromRow, PartialEq)]
struct Dog {
    id: i32,
    person_id: i32,
    name: String,
}

impl Meta for Dog {
    type Id = i32;
    type Store = PgPool;
    type Error = Error;

    fn id(&self) -> i32 {
        self.id
    }
}

struct DogInput {
    person_id: i32,
    name: String,
}

#[async_trait]
impl Create for Dog {
    type Input = DogInput;

    async fn create(input: DogInput, store: &PgPool) -> Result<Self, Error> {
        sqlx::query_as::<_, Dog>("INSERT INTO dogs (person_id, name) VALUES ($1, $2) RETURNING *")
            .bind(input.person_id)
            .bind(input.name)
            .fetch_one(store)
            .await
    }
}

#[async_trait]
impl Read for Dog {
    async fn read(id: i32, store: &PgPool) -> Result<Self, Error> {
        sqlx::query_as::<_, Dog>("SELECT * FROM dogs WHERE id = $1")
            .bind(id)
            .fetch_one(store)
            .await
    }

    async fn read_optional(id: i32, store: &PgPool) -> Result<Option<Self>, Error> {
        sqlx::query_as::<_, Dog>("SELECT * FROM dogs WHERE id = $1")
            .bind(id)
            .fetch_optional(store)
            .await
    }

    async fn read_many(ids: &[i32], store: &PgPool) -> Result<Vec<Self>, Error> {
        sqlx::query_as::<_, Dog>("SELECT * FROM dogs WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(store)
            .await
    }
}

belongs_to!(Dog, Person, "dogs", person);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    sqlx::migrate!("examples/migrations").run(&store).await?;

    let person1 = Person::create(
        PersonInput {
            name: "Person 1".into(),
        },
        &store,
    )
    .await?;
    let person2 = Person::create(
        PersonInput {
            name: "Person 2".into(),
        },
        &store,
    )
    .await?;

    let dog1 = Dog::create(
        DogInput {
            person_id: person1.id(),
            name: "Dog 1".into(),
        },
        &store,
    )
    .await?;
    let dog2 = Dog::create(
        DogInput {
            person_id: person1.id(),
            name: "Dog 2".into(),
        },
        &store,
    )
    .await?;
    let dog3 = Dog::create(
        DogInput {
            person_id: person2.id(),
            name: "Dog 3".into(),
        },
        &store,
    )
    .await?;

    assert_eq!(
        person1.children(&store).await?,
        vec![dog1.clone(), dog2.clone()]
    );
    assert_eq!(person2.children(&store).await?, vec![dog3.clone()]);

    assert_eq!(dog1.parent(&store).await?, person1);
    assert_eq!(dog2.parent(&store).await?, person1);
    assert_eq!(dog3.parent(&store).await?, person2);

    assert_eq!(
        Dog::for_parent(&person1, &store).await?,
        vec![dog1.clone(), dog2.clone()]
    );
    assert_eq!(
        Dog::for_parents(&vec![person1, person2], &store).await?,
        vec![dog1, dog2, dog3]
    );

    Ok(())
}
