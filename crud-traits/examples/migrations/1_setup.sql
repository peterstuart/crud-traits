CREATE TABLE people (
    id serial PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE dogs (
    id serial PRIMARY KEY,
    person_id int NOT NULL REFERENCES people (id),
    name text NOT NULL
);

