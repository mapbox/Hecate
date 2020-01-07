-- tokens need a unique identifier so they can be deleted
-- without having the original token
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

ALTER TABLE users_tokens
    ADD COLUMN
        id UUID UNIQUE NOT NULL;

ALTER TABLE users_tokens
    ALTER COLUMN name
        SET NOT NULL;

ALTER TABLE users_tokens
    ALTER COLUMN name
        SET NOT NULL;

ALTER TABLE users_tokens
    ALTER COLUMN uid
        SET NOT NULL;

ALTER TABLE users_tokens
    ALTER COLUMN scope
        SET NOT NULL;
