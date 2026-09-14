# Project Overview

## Database Tables

### `users`
| Column          | Type          | Description                        |
|-----------------|---------------|------------------------------------|
| `id`            | SERIAL PK     | Auto-incremented user ID           |
| `username`      | VARCHAR(20)   | Unique, 4–20 characters            |
| `email`         | TEXT          | Unique, stored lowercase           |
| `password`      | TEXT          | Argon2 hash                        |
| `bio`           | TEXT          | Optional profile bio               |
| `date_of_birth` | DATE          | Optional                           |
| `role`          | user_role     | Enum: `user` or `admin`            |
| `created_at`    | TIMESTAMPTZ   | Set automatically on insert        |

### `refresh_tokens`
| Column       | Type        | Description                              |
|--------------|-------------|------------------------------------------|
| `id`         | SERIAL PK   | Auto-incremented                         |
| `user_id`    | INTEGER FK  | References `users(id)`, cascades deletes |
| `token`      | TEXT        | SHA-256 hash of the actual token         |
| `expires_at` | TIMESTAMPTZ | Token expiry time (7 days from login)    |
| `created_at` | TIMESTAMPTZ | Set automatically on insert              |

---

## API Endpoints

| Method | Path              | Auth required | Description                                      |
|--------|-------------------|---------------|--------------------------------------------------|
| POST   | `/register`       | No            | Register a new user, returns full user profile   |
| POST   | `/login`          | No            | Login, returns access token + refresh token      |
| POST   | `/auth/refresh`   | No            | Exchange refresh token for a new access token    |
| POST   | `/auth/logout`    | Yes (Bearer)  | Revoke refresh token                             |
| GET    | `/users/{id}`     | No            | Get public profile of a user by ID               |
| DELETE | `/users/{id}`     | Yes (Bearer)  | Delete user by ID (admin only)                   |
| GET    | `/search`         | No            | Search users by username (`?username=...`)       |
