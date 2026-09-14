-- Reuse detection for refresh tokens.
--
-- Instead of deleting a refresh token on rotation, we now mark it `used = true`
-- and keep the row around. That lets us tell two cases apart on the next request:
--   * token is missing entirely        -> unknown/invalid token        -> 401
--   * token exists but `used = true`    -> the SAME token was spent twice -> compromise
-- The second case means the token leaked (a thief and the real user both hold it),
-- so we wipe every refresh token of that user and force a fresh login.
--
-- Used rows are still cleaned up once they pass `expires_at` by the existing
-- cleanup task, so the table does not grow without bound.

ALTER TABLE refresh_tokens ADD COLUMN used BOOLEAN NOT NULL DEFAULT false;
