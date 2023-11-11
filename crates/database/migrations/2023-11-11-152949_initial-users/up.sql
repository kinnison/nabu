-- Initial database setup for users for Nabu

CREATE TABLE identity (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    admin BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE token (
    id SERIAL PRIMARY KEY,
    identity INTEGER NOT NULL REFERENCES identity(id),
    title VARCHAR NOT NULL,
    content VARCHAR NOT NULL UNIQUE DEFAULT md5(gen_random_uuid()::varchar)
);
