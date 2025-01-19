-- @block Schema
-- Extensions
CREATE EXTENSION IF NOT EXISTS vector;
-- Tables
CREATE TABLE main (
    uuid UUID DEFAULT gen_random_uuid() NOT NULL,
    text TEXT NOT NULL UNIQUE,
    embedding VECTOR(3072),
    PRIMARY KEY (uuid)
);