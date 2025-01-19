-- @block Schema
-- Extensions
CREATE EXTENSION IF NOT EXISTS vector;
-- Tables
CREATE TABLE main (
    uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
    text TEXT NOT NULL UNIQUE,
    embedding VECTOR(3072) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    PRIMARY KEY (uuid)
);