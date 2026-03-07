# vais-registry-server

A REST API server for hosting and distributing Vais packages. Built with Axum, SQLite, and tower-http.

## Features

- Package publishing, retrieval, and version management
- User authentication with scoped API tokens (Argon2 + SHA-256)
- Full-text search with category/keyword filtering and sort options
- Semver-aware dependency resolution
- Package signing with Ed25519
- Version yanking/unyanking
- Multi-owner package management
- Web UI dashboard (HTML/CSS, no JS framework)
- CORS configuration (per-origin or allow-all)
- TOML file or environment variable configuration

## Quick Start

### Prerequisites

- Rust 1.70+ (edition 2021)
- SQLite 3

### Build & Run

```bash
# Build
cargo build -p vais-registry-server --release

# Run with defaults (0.0.0.0:3000, SQLite at ./data/registry.db)
cargo run -p vais-registry-server

# Run with environment variables
VAIS_REGISTRY_PORT=8080 VAIS_REGISTRY_DB=./my.db cargo run -p vais-registry-server

# Run with config file
cargo run -p vais-registry-server -- --config registry.toml
```

### Configuration

Configuration is loaded from environment variables or a TOML file.

#### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `VAIS_REGISTRY_HOST` | `0.0.0.0` | Bind host address |
| `VAIS_REGISTRY_PORT` | `3000` | Bind port (also reads `PORT` for Fly.io) |
| `VAIS_REGISTRY_DB` | `./data/registry.db` | SQLite database path |
| `VAIS_REGISTRY_STORAGE` | `./data/packages` | Package tarball storage directory |
| `VAIS_REGISTRY_MAX_UPLOAD` | `52428800` | Max upload size in bytes (50 MB) |
| `VAIS_REGISTRY_TOKEN_EXPIRY` | `365` | Default token expiration in days |
| `VAIS_REGISTRY_CORS_ALL` | `false` | Allow all CORS origins (`true`/`1`) |
| `VAIS_REGISTRY_CORS_ORIGINS` | (empty) | Comma-separated allowed origins |
| `VAIS_REGISTRY_LOGGING` | `true` | Enable request tracing (`false`/`0` to disable) |
| `VAIS_REGISTRY_ADMIN_USER` | (none) | Initial admin username |
| `VAIS_REGISTRY_ADMIN_PASS` | (none) | Initial admin password |

#### TOML Config File

```toml
host = "0.0.0.0"
port = 3000
database_path = "./data/registry.db"
storage_path = "./data/packages"
max_upload_size = 52428800
token_expiration_days = 365
cors_allow_all = false
cors_origins = []
enable_logging = true
admin_username = "admin"
admin_password = "changeme"
```

## API Reference

Base URL: `/api/v1`

### Health

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | No | Health check (root level) |
| `GET` | `/api/v1/health` | No | Health check (API level) |

**Response:**
```json
{ "status": "ok", "version": "0.1.0" }
```

### Authentication

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/api/v1/auth/register` | No | Create a new user account |
| `POST` | `/api/v1/auth/login` | No | Log in and receive a bearer token |
| `GET` | `/api/v1/auth/me` | Bearer | Get current user info |
| `GET` | `/api/v1/auth/tokens` | Bearer | List API tokens |
| `POST` | `/api/v1/auth/tokens` | Bearer | Create a new API token |
| `DELETE` | `/api/v1/auth/tokens/:id` | Bearer | Revoke an API token |

**Register:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secret123", "email": "alice@example.com"}'
```

**Login:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secret123"}'
```

Response:
```json
{
  "token": "vais_xxxxxxxxxxxx",
  "expires_at": "2027-03-07T00:00:00Z",
  "user": { "id": "...", "username": "alice", "email": "alice@example.com", "is_admin": false }
}
```

**Create Token:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/tokens \
  -H "Authorization: Bearer vais_xxxxxxxxxxxx" \
  -H "Content-Type: application/json" \
  -d '{"name": "ci-deploy", "scopes": ["publish"], "expires_in_days": 90}'
```

Token scopes: `publish`, `yank`, `admin`.

### Packages

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/api/v1/packages/publish` | Bearer (`publish`) | Publish a package version |
| `GET` | `/api/v1/packages/:name` | No | Get package metadata + all versions |
| `GET` | `/api/v1/packages/:name/:version` | No | Download a specific version tarball |
| `POST` | `/api/v1/packages/:name/:version/yank` | Bearer (`yank`) | Yank a version |
| `POST` | `/api/v1/packages/:name/:version/unyank` | Bearer (`yank`) | Unyank a version |

**Publish:**
```bash
curl -X POST http://localhost:3000/api/v1/packages/publish \
  -H "Authorization: Bearer vais_xxxxxxxxxxxx" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-package",
    "version": "1.0.0",
    "description": "A useful Vais library",
    "license": "MIT",
    "keywords": ["utility"],
    "categories": ["algorithms"],
    "dependencies": {
      "std-io": "^1.0",
      "serde": {"version": ">=2.0", "features": ["derive"], "optional": true}
    }
  }'
```

**Get Package Info:**
```bash
curl http://localhost:3000/api/v1/packages/my-package
```

Response:
```json
{
  "name": "my-package",
  "description": "A useful Vais library",
  "license": "MIT",
  "keywords": ["utility"],
  "categories": ["algorithms"],
  "downloads": 42,
  "versions": [
    {
      "version": "1.0.0",
      "checksum": "sha256:...",
      "size": 1024,
      "yanked": false,
      "downloads": 42,
      "dependencies": [
        { "name": "std-io", "version_req": "^1.0", "kind": "normal" }
      ]
    }
  ],
  "owners": ["alice"]
}
```

### Search & Discovery

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/v1/search?q=...` | No | Search packages |
| `GET` | `/api/v1/categories` | No | List all categories |
| `GET` | `/api/v1/categories/:category` | No | Browse packages in a category |
| `GET` | `/api/v1/popular` | No | Most downloaded packages |
| `GET` | `/api/v1/recent` | No | Recently updated packages |

**Search Query Parameters:**

| Param | Default | Description |
|---|---|---|
| `q` | (required) | Search query string |
| `limit` | `20` | Max results per page |
| `offset` | `0` | Pagination offset |
| `sort` | `downloads` | Sort order: `downloads`, `newest`, `name`, `relevance` |
| `category` | (none) | Filter by category |
| `keyword` | (none) | Filter by keyword |

### Index (Client-Compatible)

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/v1/index.json` | No | Full registry index |
| `GET` | `/api/v1/packages/:name/index.json` | No | Per-package index entry |

### Users & Owners

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/v1/users/:username` | No | Get public user profile |
| `POST` | `/api/v1/packages/:name/owners` | Bearer | Add a package owner |
| `DELETE` | `/api/v1/packages/:name/owners/:username` | Bearer | Remove a package owner |

### Web UI

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | Registry homepage |
| `GET` | `/dashboard` | User dashboard |
| `GET` | `/packages/:name` | Package detail page |
| `GET` | `/static/styles.css` | Stylesheet |

## Deployment

### Docker

```dockerfile
FROM rust:1.70-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build -p vais-registry-server --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/vais-registry-server /usr/local/bin/
ENV VAIS_REGISTRY_HOST=0.0.0.0
ENV VAIS_REGISTRY_PORT=3000
EXPOSE 3000
CMD ["vais-registry-server"]
```

### Fly.io

```bash
fly launch --name vais-registry
fly secrets set VAIS_REGISTRY_ADMIN_USER=admin VAIS_REGISTRY_ADMIN_PASS=changeme
fly deploy
```

Fly.io automatically sets `PORT`, which the server reads as a fallback for `VAIS_REGISTRY_PORT`.

### Persistent Storage

The server stores data in two locations:
- **SQLite database** (`database_path`): User accounts, package metadata, tokens
- **Package storage** (`storage_path`): Uploaded tarballs

For production, mount persistent volumes at these paths. Example for Fly.io:

```bash
fly volumes create vais_data --size 10 --region sea
# In fly.toml: [mounts] source = "vais_data" destination = "/data"
```

## Architecture

```
Request -> Axum Router -> Handler -> DB (SQLite) / Storage (filesystem)
                       |
                       +-> AuthUser extractor (Bearer token -> SHA-256 hash -> DB lookup)
                       +-> CORS (tower-http)
                       +-> Tracing (tower-http)
```

### Module Structure

| Module | Description |
|---|---|
| `config.rs` | Server configuration (env vars, TOML, defaults) |
| `db.rs` | SQLite database operations (sqlx) |
| `error.rs` | Error types and HTTP error responses |
| `handlers/` | HTTP request handlers (auth, packages, index, users, web) |
| `models.rs` | Data models and request/response types |
| `router.rs` | Route definitions and middleware setup |
| `semver_resolve.rs` | Semantic version resolution |
| `signing.rs` | Ed25519 package signing |
| `storage.rs` | Package tarball filesystem storage |

## License

MIT
