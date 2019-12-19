-- Null access values are no longer allowed
UPDATE users
    SET access = 'default'
    WHERE access IS NULL;
