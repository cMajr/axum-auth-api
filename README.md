# axum-auth-api

REST API in Rust with user registration, JWT authentication and refresh token rotation.

Stack: Axum 0.8, SQLx 0.9, PostgreSQL, Argon2, jsonwebtoken.

## Features

- Passwords hashed with Argon2
- Short-lived access tokens (JWT, 15 min) and refresh tokens (7 days)
- Refresh tokens are stored as SHA-256 hashes and rotated on every use
- Reusing an old refresh token revokes all sessions of that user
- Rate limiting: 30 requests/min per IP overall; login is limited to 5 attempts per email and 50 per IP within 5 minutes
- Request validation with readable error messages
- Expired refresh tokens are removed by a background task
- Migrations run on startup

## Running locally

Start PostgreSQL:

```sh
docker run -d --name axum-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=axum -p 5432:5432 postgres:16
```

Create `.env`:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/axum
JWT_SECRET=change-me
BIND_ADDR=127.0.0.1:3000
```

SQLx checks queries against the database at compile time, so apply the migrations before building:

```sh
cargo install sqlx-cli
sqlx migrate run
cargo run
```

| Variable       | Required | Default        |
|----------------|----------|----------------|
| `DATABASE_URL` | yes      |                |
| `JWT_SECRET`   | yes      |                |
| `BIND_ADDR`    | no       | `0.0.0.0:3000` |
| `RUST_LOG`     | no       | `info`         |

## Endpoints

| Method | Path            | Auth   | Description                                  |
|--------|-----------------|--------|----------------------------------------------|
| POST   | `/register`     |        | Create an account                            |
| POST   | `/login`        |        | Get access and refresh tokens                |
| POST   | `/auth/refresh` |        | Exchange a refresh token for a new pair      |
| POST   | `/auth/logout`  | Bearer | Revoke a refresh token                       |
| POST   | `/update/me`    | Bearer | Update username, bio or date of birth        |
| GET    | `/users/{id}`   |        | Public profile                               |
| GET    | `/search`       |        | Search users by username (`?username=`, 4–20 chars) |
| DELETE | `/users/{id}`   | Admin  | Delete a user                                |

There is no endpoint for creating admins. To make a user an admin:

```sql
UPDATE users SET role = 'admin' WHERE email = 'you@example.com';
```

## Example

```sh
curl -X POST localhost:3000/register -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"password123"}'

curl -X POST localhost:3000/login -H 'Content-Type: application/json' \
  -d '{"email":"alice@example.com","password":"password123"}'
# {"access_token":"...","refresh_token":"..."}

curl -X POST localhost:3000/auth/refresh -H 'Content-Type: application/json' \
  -d '{"token":"<refresh_token>"}'
```

Errors are returned as `{"error": "..."}` with the matching HTTP status.
