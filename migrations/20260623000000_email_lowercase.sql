-- Normalize emails to lowercase and enforce case-insensitive uniqueness.

UPDATE users SET email = lower(email);

ALTER TABLE users DROP CONSTRAINT users_email_key;

CREATE UNIQUE INDEX users_email_lower_key ON users (lower(email));
