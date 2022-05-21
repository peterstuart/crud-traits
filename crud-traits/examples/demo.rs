use async_trait::async_trait;
use crud_traits::{Create, MappedModel, MappedModelWithParentId, Meta};
use crud_traits_macros::{belongs_to, has_many, has_one, Read};
use sqlx::{Error, FromRow, PgPool};
use std::env;

#[derive(Clone, Debug, Eq, Read, FromRow, PartialEq)]
#[has_many(child = "Dog")]
#[crud(table = "people")]
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

#[derive(Clone, Debug, Eq, Read, FromRow, PartialEq)]
#[belongs_to(parent = "Person", plural_alias = "people", table = "dogs")]
#[has_one(child = "Bed")]
#[crud(table = "dogs")]
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

#[derive(Clone, Debug)]
struct MappedDog {
    dog: Dog,
}

#[async_trait]
impl MappedModel for MappedDog {
    type OriginalModel = Dog;
    type Error = Error;

    fn id(&self) -> i32 {
        self.dog.id
    }

    async fn from_model(dog: Dog, _: &PgPool) -> Result<Self, Error> {
        Ok(Self { dog })
    }

    async fn from_models(dogs: Vec<Dog>, _: &PgPool) -> Result<Vec<Self>, Error> {
        Ok(dogs.into_iter().map(|dog| Self { dog }).collect())
    }
}

impl MappedModelWithParentId<Person> for MappedDog {
    fn parent_id(&self) -> i32 {
        self.dog.person_id
    }
}

#[derive(Clone, Debug, Eq, Read, FromRow, PartialEq)]
#[belongs_to(parent = "Dog", table = "beds")]
#[crud(table = "beds")]
struct Bed {
    id: i32,
    dog_id: i32,
    location: String,
}

impl Meta for Bed {
    type Id = i32;
    type Store = PgPool;
    type Error = Error;

    fn id(&self) -> i32 {
        self.id
    }
}

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
        person1.dogs(&store).await?,
        vec![dog1.clone(), dog2.clone()]
    );
    assert_eq!(person2.dogs(&store).await?, vec![dog3.clone()]);

    assert_eq!(dog1.person(&store).await?, person1);
    assert_eq!(dog2.person(&store).await?, person1);
    assert_eq!(dog3.person(&store).await?, person2);

    assert_eq!(
        Dog::for_person(&person1, &store).await?,
        vec![dog1.clone(), dog2.clone()]
    );

    let dogs_by_person_ids = Dog::for_people(&vec![person1.id, person2.id], &store).await?;
    assert_eq!(dogs_by_person_ids.get(&person1.id), Some(&vec![dog1, dog2]));
    assert_eq!(dogs_by_person_ids.get(&person2.id), Some(&vec![dog3]));

    Ok(())
}
