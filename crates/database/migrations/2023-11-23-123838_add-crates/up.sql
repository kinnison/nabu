-- Crates with basic content

CREATE TABLE krate (
    id SERIAL NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    owner INTEGER NOT NULL REFERENCES identity(id)
);

CREATE TABLE kratever (
    id SERIAL NOT NULL PRIMARY KEY,
    krate INTEGER NOT NULL REFERENCES krate(id),
    exposed BOOLEAN NOT NULL,
    ver VARCHAR NOT NULL,
    yanked BOOLEAN NOT NULL,
    metadata JSONB NOT NULL,

    CONSTRAINT version_unique UNIQUE(krate, ver)
);

