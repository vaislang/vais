# Vais Package Registry Deployment Guide

This guide covers deploying the Vais package registry server using Docker.

## Quick Start with Docker

### Build the Registry Image

```bash
docker build -f Dockerfile.registry -t vais-registry:latest .
```

### Run the Registry Server

```bash
docker run -d \
  --name vais-registry \
  -p 3000:3000 \
  -v vais-registry-data:/data \
  -e VAIS_REGISTRY_ADMIN_USER=admin \
  -e VAIS_REGISTRY_ADMIN_PASS=changeme \
  vais-registry:latest
```

The registry will be available at `http://localhost:3000`.

## Environment Variables

Configure the registry using these environment variables:

- `VAIS_REGISTRY_HOST` - Server bind address (default: `0.0.0.0`)
- `VAIS_REGISTRY_PORT` - Server port (default: `3000`)
- `VAIS_REGISTRY_DB` - SQLite database path (default: `/data/registry.db`)
- `VAIS_REGISTRY_STORAGE` - Package storage path (default: `/data/packages`)
- `VAIS_REGISTRY_ADMIN_USER` - Initial admin username (optional)
- `VAIS_REGISTRY_ADMIN_PASS` - Initial admin password (optional)
- `RUST_LOG` - Logging level (default: `vais_registry_server=info`)

## Using the Registry

### 1. Login to the Registry

```bash
vaisc pkg login --registry http://localhost:3000
# Enter username and password when prompted
```

### 2. Publish a Package

From your package directory (containing `vais.toml`):

```bash
vaisc pkg publish --registry http://localhost:3000
```

**Features:**
- Automatically reads package metadata from `vais.toml`
- Creates `.tar.gz` archive excluding `.git`, `target`, `node_modules`
- Computes SHA-256 checksum
- Uploads package with multipart form data
- Verifies checksum after upload

**Dry run:**
```bash
vaisc pkg publish --registry http://localhost:3000 --dry-run
```

### 3. Search for Packages

```bash
vaisc pkg search <query> --registry http://localhost:3000
```

### 4. Install Packages

```bash
# Install specific package
vaisc pkg install <package-name> --registry http://localhost:3000

# Install with version constraint
vaisc pkg install json-parser@^1.0.0

# Install from vais.toml dependencies
vaisc pkg install
```

**Features:**
- Semver dependency resolution (supports `^`, `~`, `>=`, exact versions)
- Transitive dependency resolution
- Package caching in `~/.vais/registry/cache/`
- Lock file generation (`vais.lock`)
- Offline mode with cached packages

### 5. Update Dependencies

```bash
# Update all dependencies
vaisc pkg update

# Update specific package
vaisc pkg update <package-name>
```

## Semver Version Constraints

The registry supports standard semantic versioning constraints:

| Constraint | Example | Meaning |
|------------|---------|---------|
| Exact | `1.2.3` or `=1.2.3` | Exactly version 1.2.3 |
| Caret | `^1.2.3` | >=1.2.3, <2.0.0 (default) |
| Tilde | `~1.2.3` | >=1.2.3, <1.3.0 |
| Greater/Less | `>=1.2.0`, `<2.0.0` | Range constraints |
| Wildcard | `1.*`, `1.2.*` | Any matching version |
| Multiple | `>=1.0.0, <2.0.0` | Comma-separated constraints |

## Package Structure

A valid Vais package requires:

```
my-package/
├── vais.toml          # Package manifest
├── src/               # Source code
│   └── lib.vais
└── README.md          # (optional)
```

### vais.toml Example

```toml
[package]
name = "my-package"
version = "1.0.0"
authors = ["Your Name <your.email@example.com>"]
description = "A sample Vais package"
license = "MIT"

[dependencies]
json-parser = "^1.0.0"
http-client = { version = "~2.1.0" }
local-lib = { path = "../local-lib" }

[dev-dependencies]
test-framework = ">=0.5.0"
```

## Production Deployment

### Using Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  registry:
    build:
      context: .
      dockerfile: Dockerfile.registry
    ports:
      - "3000:3000"
    volumes:
      - registry-data:/data
    environment:
      - VAIS_REGISTRY_HOST=0.0.0.0
      - VAIS_REGISTRY_PORT=3000
      - VAIS_REGISTRY_ADMIN_USER=admin
      - VAIS_REGISTRY_ADMIN_PASS=${ADMIN_PASSWORD}
      - RUST_LOG=vais_registry_server=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  registry-data:
```

Run with:
```bash
export ADMIN_PASSWORD="your-secure-password"
docker-compose up -d
```

### Behind a Reverse Proxy (nginx)

```nginx
server {
    listen 80;
    server_name registry.vais.dev;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Increase timeouts for large package uploads
        proxy_connect_timeout 300s;
        proxy_send_timeout 300s;
        proxy_read_timeout 300s;
        client_max_body_size 100M;
    }
}
```

## Security

### Package Signing

All packages are verified with SHA-256 checksums:
- Computed during `vaisc pkg publish`
- Stored in registry metadata
- Verified during `vaisc pkg install`
- Re-verified after upload (as of Phase 33 Stage 3)

### Authentication

The registry uses bearer token authentication:
- Tokens are stored in `~/.vais/credentials.toml` after login
- All publish/yank operations require authentication
- Token is sent via `Authorization: Bearer <token>` header

### Best Practices

1. **Use HTTPS in production** - Always deploy behind TLS/SSL
2. **Set strong admin password** - Use `VAIS_REGISTRY_ADMIN_PASS` env var
3. **Regular backups** - Backup `/data` volume regularly
4. **Monitor logs** - Use `RUST_LOG` for debugging
5. **Restrict publish access** - Only trusted users should have publish tokens

## Troubleshooting

### Package Not Found

```bash
# Update registry index
vaisc pkg install --no-offline <package>
```

### Checksum Mismatch

If you see checksum warnings during install:
1. Check network connectivity
2. Try clearing cache: `rm -rf ~/.vais/registry/cache/<package>`
3. Re-download with `vaisc pkg install --update`

### Authentication Errors

```bash
# Re-login to refresh token
vaisc pkg login --registry <url>
```

### Database Locked

If SQLite database is locked:
```bash
docker restart vais-registry
```

## API Endpoints

The registry server exposes these HTTP endpoints:

- `GET /health` - Health check
- `GET /index.json` - Package index
- `GET /packages/:name/:version` - Package metadata
- `POST /packages/publish` - Publish package (auth required)
- `POST /packages/:name/:version/yank` - Yank package (auth required)
- `POST /auth/login` - User login
- `POST /auth/register` - User registration

## Monitoring

Check registry status:

```bash
curl http://localhost:3000/health
```

View logs:
```bash
docker logs -f vais-registry
```

## Data Persistence

The registry stores data in `/data` volume:

```
/data/
├── registry.db          # SQLite database (users, packages metadata)
└── packages/            # Package archives
    ├── json-parser/
    │   └── 1.0.0.tar.gz
    └── http-client/
        └── 2.1.0.tar.gz
```

**Backup:**
```bash
docker run --rm -v vais-registry-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/registry-backup.tar.gz /data
```

**Restore:**
```bash
docker run --rm -v vais-registry-data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/registry-backup.tar.gz -C /
```

## Development

Run registry locally without Docker:

```bash
cd crates/vais-registry-server
cargo run --release
```

Set environment variables in `.env` file:
```bash
VAIS_REGISTRY_HOST=127.0.0.1
VAIS_REGISTRY_PORT=3000
VAIS_REGISTRY_DB=./data/registry.db
VAIS_REGISTRY_STORAGE=./data/packages
VAIS_REGISTRY_ADMIN_USER=admin
VAIS_REGISTRY_ADMIN_PASS=admin123
```

## Migration from Other Registries

To migrate packages from another registry:

1. Download all packages from old registry
2. Set up new registry
3. Re-publish packages using `vaisc pkg publish`

## License

The Vais registry server is MIT licensed. See LICENSE for details.
