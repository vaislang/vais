# Vais Registry Quick Start

Quick reference for common package registry operations.

## Deploy Registry

```bash
# Quick deploy (uses default password 'changeme')
./scripts/deploy-registry.sh

# Deploy with custom admin password
./scripts/deploy-registry.sh my-secure-password

# Or with Docker Compose
docker-compose -f docker-compose.registry.yml up -d
```

Registry will be available at: `http://localhost:3000`

## Package Author Commands

### Login
```bash
vaisc pkg login --registry http://localhost:3000
# Username: admin
# Password: [your-password]
```

### Publish Package
```bash
# From your package directory
vaisc pkg publish --registry http://localhost:3000

# With verbose output
vaisc pkg publish --registry http://localhost:3000 --verbose

# Dry run (validate without uploading)
vaisc pkg publish --registry http://localhost:3000 --dry-run
```

### Yank Package
```bash
vaisc pkg yank <package-name> <version> --registry http://localhost:3000
```

## Package User Commands

### Search Packages
```bash
vaisc pkg search <query> --registry http://localhost:3000
```

### Install Package
```bash
# Latest version
vaisc pkg install <package-name> --registry http://localhost:3000

# Specific version
vaisc pkg install <package-name>@1.2.3

# Version constraint
vaisc pkg install <package-name>@^1.0.0

# Install from vais.toml
vaisc pkg install
```

### Update Dependencies
```bash
# Update all
vaisc pkg update

# Update specific package
vaisc pkg update <package-name>
```

## Version Constraints Quick Reference

| Constraint | Example | Matches |
|------------|---------|---------|
| Caret (default) | `^1.2.3` | >=1.2.3, <2.0.0 |
| Tilde | `~1.2.3` | >=1.2.3, <1.3.0 |
| Exact | `=1.2.3` or `1.2.3` | Exactly 1.2.3 |
| Greater/Equal | `>=1.2.0` | 1.2.0 and above |
| Less | `<2.0.0` | Below 2.0.0 |
| Wildcard | `1.*` | Any 1.x.x |
| Multiple | `>=1.0.0, <2.0.0` | Between 1.0.0 and 2.0.0 |

## Package Structure

```
my-package/
├── vais.toml          # Required
├── src/               # Required
│   └── lib.vais      # or main.vais
└── README.md          # Optional
```

## Minimal vais.toml

```toml
[package]
name = "my-package"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]
description = "Package description"
license = "MIT"

[dependencies]
# dependency-name = "^1.0.0"
```

## Docker Registry Management

```bash
# View logs
docker logs -f vais-registry

# Stop
docker stop vais-registry

# Restart
docker restart vais-registry

# Remove
docker rm -f vais-registry

# Backup data
docker run --rm -v vais-registry-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/registry-backup.tar.gz /data

# Restore data
docker run --rm -v vais-registry-data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/registry-backup.tar.gz -C /
```

## Troubleshooting

### Package not found
```bash
# Update index
vaisc pkg install <package> --no-offline
```

### Authentication failed
```bash
# Re-login
vaisc pkg login --registry http://localhost:3000
```

### Registry not responding
```bash
# Check status
docker ps | grep vais-registry
docker logs vais-registry

# Restart
docker restart vais-registry
```

### Clear cache
```bash
rm -rf ~/.vais/registry/cache
```

## Environment Variables

```bash
# Default registry URL
export VAIS_REGISTRY_URL=http://localhost:3000

# Admin credentials (for deployment)
export ADMIN_USER=admin
export ADMIN_PASSWORD=secure-password
```

## Common Workflows

### Publish New Package
```bash
# 1. Create package
mkdir my-package && cd my-package
vaisc pkg init --name my-package

# 2. Edit vais.toml and add code
# ...

# 3. Login to registry
vaisc pkg login --registry http://localhost:3000

# 4. Publish
vaisc pkg publish --registry http://localhost:3000
```

### Use Package in Another Project
```bash
# 1. Add to vais.toml
echo 'my-package = "^0.1.0"' >> vais.toml

# 2. Install
vaisc pkg install

# 3. Import in code
# import my_package::{...}
```

### Update Package Version
```bash
# 1. Edit version in vais.toml
# version = "0.2.0"

# 2. Publish new version
vaisc pkg publish --registry http://localhost:3000
```

## Full Documentation

For complete documentation, see:
- Deployment Guide: `REGISTRY_DEPLOYMENT.md`
- Technical Summary: `docs/phase33-stage3-summary.md`
- Example Package: `examples/package/`

## Support

- Issues: GitHub Issues
- Validation: `./scripts/validate-phase33-stage3.sh`
- Health Check: `curl http://localhost:3000/health`
