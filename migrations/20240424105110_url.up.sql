-- Add up migration script here
CREATE TABLE url (
    id VARCHAR(4) PRIMARY KEY,
    long_url VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
