-- POSTGRESS TABLE EXAMPLE

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS
    "oauth" (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    state VARCHAR(255) NOT NULL,
    verifier VARCHAR(255) NOT NULL
);