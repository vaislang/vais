# Production Deployment Guide

This guide covers deploying Vais applications to production environments with best practices for security, monitoring, and reliability.

## Table of Contents

- [Production Checklist](#production-checklist)
- [Docker Deployment Guide](#docker-deployment-guide)
- [CI/CD Pipeline Template](#cicd-pipeline-template)
- [Tutorial: REST API to Production](#tutorial-rest-api-to-production)

---

## Production Checklist

Before deploying your Vais application to production, ensure you have addressed these critical areas.

### Security

#### Ownership Checking

Vais provides compile-time memory safety through ownership checking. Choose the appropriate level for your use case:

```bash
# Strict ownership checking (recommended for new code)
vaisc build src/ -o server --strict-ownership

# Warn-only mode (for migration)
vaisc build src/ -o server --warn-only-ownership

# Disable checks (legacy code only, not recommended)
vaisc build src/ -o server --no-ownership-check
```

**Production recommendation**: Always use `--strict-ownership` for new applications. This prevents use-after-move, double-free, and other memory safety issues at compile time.

#### Input Validation

Always validate user input before processing:

```vais
# Validate string length
F validate_username(name: str) -> bool {
    len := strlen(name)
    len >= 3 && len <= 32
}

# Sanitize path inputs
F is_safe_path(path: str) -> bool {
    # Prevent path traversal
    I str_contains(path, "..") { R false }
    I str_contains(path, "//") { R false }
    true
}

# Validate numeric ranges
F validate_port(port: i64) -> bool {
    port >= 1024 && port <= 65535
}
```

#### Hardcoded Secret Detection

Use the security analyzer to detect hardcoded secrets:

```bash
# Run security audit
vaisc security-scan src/ --output audit.json

# Check for hardcoded secrets, SQL injection, XSS
vaisc security-scan src/ --rules secrets,sql-injection,xss
```

See [SECURITY_AUDIT_SUMMARY.md](SECURITY_AUDIT_SUMMARY.md) for comprehensive security guidelines.

### Monitoring

#### Logging

Implement structured logging for production observability:

```vais
U std/time
U std/file

S LogLevel { Debug, Info, Warn, Error }

S Logger {
    file_path: str,
    min_level: LogLevel
}

X Logger {
    F new(path: str, level: LogLevel) -> Logger {
        Logger { file_path: path, min_level: level }
    }

    F log(&self, level: LogLevel, msg: str) -> i64 {
        I self.should_log(level) {
            timestamp := time::now()
            level_str := M level {
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warn => "WARN",
                LogLevel::Error => "ERROR"
            }

            line := format_log(timestamp, level_str, msg)
            file::append(self.file_path, line)
        }
        0
    }

    F should_log(&self, level: LogLevel) -> bool {
        level as i64 >= self.min_level as i64
    }
}

# Usage
F main() -> i64 {
    logger := Logger::new("/var/log/app.log", LogLevel::Info)
    logger.log(LogLevel::Info, "Server starting")
    logger.log(LogLevel::Debug, "This won't be logged")
    0
}
```

#### Metrics Collection

Track key application metrics:

```vais
U std/time

S Metrics {
    requests_total: i64,
    requests_failed: i64,
    response_time_sum: i64,
    active_connections: i64
}

X Metrics {
    F new() -> Metrics {
        Metrics {
            requests_total: 0,
            requests_failed: 0,
            response_time_sum: 0,
            active_connections: 0
        }
    }

    F record_request(&self, duration_ms: i64, success: bool) -> i64 {
        self.requests_total = self.requests_total + 1
        I !success {
            self.requests_failed = self.requests_failed + 1
        }
        self.response_time_sum = self.response_time_sum + duration_ms
        0
    }

    F avg_response_time(&self) -> i64 {
        I self.requests_total == 0 { R 0 }
        self.response_time_sum / self.requests_total
    }

    # Export metrics in Prometheus format
    F to_prometheus(&self) -> str {
        format_metrics(
            self.requests_total,
            self.requests_failed,
            self.avg_response_time(),
            self.active_connections
        )
    }
}
```

#### Health Checks

Implement health check endpoints:

```vais
# Health check endpoint
F handle_health(app: &App) -> Response {
    status := check_dependencies()
    I status.healthy {
        Response::json(200, "{\"status\":\"ok\"}")
    } E {
        Response::json(503, "{\"status\":\"degraded\",\"error\":\"" + status.error + "\"}")
    }
}

# Dependency health check
F check_dependencies() -> HealthStatus {
    db_ok := check_database_connection()
    cache_ok := check_cache_connection()

    I db_ok && cache_ok {
        HealthStatus { healthy: true, error: "" }
    } E {
        HealthStatus { healthy: false, error: "DB or cache unavailable" }
    }
}
```

### Build Optimization

#### Optimization Levels

```bash
# Development build (fast compile, no optimization)
vaisc build src/ -o server --optimize 0

# Production build (maximum optimization)
vaisc build src/ -o server --optimize 2

# Size-optimized build (for containers)
vaisc build src/ -o server --optimize 2 --size
```

#### Multi-file Projects

For large projects, use directory-based builds:

```bash
# Project structure
# src/
#   main.vais
#   routes/
#     users.vais
#     posts.vais
#   models/
#     user.vais
#     post.vais

# Build entire directory
vaisc build src/ -o server --optimize 2

# The compiler will:
# 1. Discover all .vais files
# 2. Resolve imports
# 3. Generate single LLVM IR
# 4. Compile to native binary
```

#### Package Dependencies

Manage dependencies with `vais.toml`:

```toml
[package]
name = "api-server"
version = "1.0.0"
authors = ["Your Name <you@example.com>"]

[dependencies]
json-parser = "^2.0.0"
http-client = "~1.5.0"
logger = "^1.0.0"

[native-dependencies]
# System libraries to link
openssl = { version = "1.1", system = true }
```

Install dependencies before building:

```bash
# Install all dependencies from registry
vaisc pkg install

# Install specific package
vaisc pkg install json-parser

# Update dependencies
vaisc pkg update
```

### Rollback Strategy

#### Version Management

```bash
# Tag releases
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0

# Build versioned artifacts
vaisc build src/ -o server-v1.0.0 --optimize 2
```

#### Blue-Green Deployment

```bash
# Deploy new version to green environment
docker tag app:latest app:blue
docker tag app:v1.0.1 app:green

# Switch traffic (in load balancer)
# If issues occur, switch back to blue
```

#### Database Migrations

Always make migrations backward-compatible:

```vais
# Good: Add nullable column
ALTER TABLE users ADD COLUMN phone TEXT NULL;

# Bad: Remove column immediately
# Instead: Deprecate → Stop writing → Remove in next release
```

---

## Docker Deployment Guide

### Multi-Stage Dockerfile

Create an optimized Docker image with multi-stage build:

```dockerfile
# Stage 1: Build environment
FROM debian:bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    llvm-17-dev \
    clang-17 \
    libclang-17-dev \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build Vais compiler
WORKDIR /build
COPY . .
RUN cargo build --release --bin vaisc

# Install vaisc to known location
RUN cp target/release/vaisc /usr/local/bin/vaisc

# Copy your application source
COPY ./app /app
WORKDIR /app

# Install Vais package dependencies
RUN vaisc pkg install

# Compile Vais application with maximum optimization
RUN vaisc build src/ -o server --optimize 2

# Stage 2: Runtime environment (slim)
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy compiled binary from builder
COPY --from=builder /app/server /usr/local/bin/server

# Copy standard library C runtime files (if needed)
COPY --from=builder /app/std/*.c /usr/local/lib/vais/

# Set up application directory
WORKDIR /app
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/server", "--health-check"] || exit 1

# Expose port
EXPOSE 8080

# Run the server
CMD ["/usr/local/bin/server"]
```

### Docker Compose

For local development and testing:

```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - ENV=production
      - LOG_LEVEL=info
      - DATABASE_URL=postgresql://postgres:password@db:5432/myapp
      - REDIS_URL=redis://cache:6379
    depends_on:
      - db
      - cache
    restart: unless-stopped
    networks:
      - app-network
    volumes:
      - ./logs:/var/log/app

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=myapp
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - app-network
    restart: unless-stopped

  cache:
    image: redis:7-alpine
    networks:
      - app-network
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - app
    networks:
      - app-network
    restart: unless-stopped

volumes:
  postgres-data:

networks:
  app-network:
    driver: bridge
```

### Nginx Configuration

Reverse proxy configuration for production:

```nginx
upstream app_backend {
    server app:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name example.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000" always;

    # Request limits
    client_max_body_size 10M;
    client_body_timeout 30s;
    client_header_timeout 30s;

    location / {
        proxy_pass http://app_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /health {
        proxy_pass http://app_backend/health;
        access_log off;
    }
}
```

### Building and Running

```bash
# Build image
docker build -t myapp:latest .

# Run with docker-compose
docker-compose up -d

# View logs
docker-compose logs -f app

# Scale horizontally
docker-compose up -d --scale app=3

# Stop all services
docker-compose down

# Clean rebuild
docker-compose down -v
docker-compose build --no-cache
docker-compose up -d
```

---

## CI/CD Pipeline Template

### GitHub Actions Workflow

Create `.github/workflows/deploy.yml`:

```yaml
name: Build and Deploy

on:
  push:
    branches: [main, staging]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install LLVM 17
        run: |
          wget https://apt.llvm.org/llvm.sh
          chmod +x llvm.sh
          sudo ./llvm.sh 17
          sudo apt-get install -y llvm-17-dev libclang-17-dev clang-17

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Build Vais compiler
        run: cargo build --release --bin vaisc

      - name: Install vaisc
        run: sudo cp target/release/vaisc /usr/local/bin/vaisc

      - name: Run Vais tests
        run: cargo test --workspace

      - name: Security audit
        run: |
          vaisc security-scan app/src/ --output security-audit.json
          # Fail if critical issues found
          if grep -q '"severity":"CRITICAL"' security-audit.json; then
            echo "Critical security issues found"
            exit 1
          fi

      - name: Lint with clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Build application
        working-directory: ./app
        run: |
          vaisc pkg install
          vaisc build src/ -o server --optimize 2 --strict-ownership

  build-and-push:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name == 'push'

    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=sha,prefix={{branch}}-
            type=semver,pattern={{version}}

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=registry,ref=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:buildcache
          cache-to: type=registry,ref=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:buildcache,mode=max

  deploy-staging:
    needs: build-and-push
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/staging'
    environment:
      name: staging
      url: https://staging.example.com

    steps:
      - name: Deploy to staging
        run: |
          # Example: Deploy to Kubernetes
          # kubectl set image deployment/app app=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:staging-${{ github.sha }}

          # Example: Deploy to AWS ECS
          # aws ecs update-service --cluster staging --service app --force-new-deployment

          # Example: SSH to server
          # ssh user@staging.example.com "docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:staging-${{ github.sha }} && docker-compose up -d"

          echo "Deployed to staging"

  deploy-production:
    needs: build-and-push
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment:
      name: production
      url: https://example.com

    steps:
      - name: Deploy to production
        run: |
          # Example: Blue-green deployment
          # 1. Deploy to green environment
          # 2. Run smoke tests
          # 3. Switch traffic from blue to green
          # 4. Keep blue as rollback target

          echo "Deployed to production"

      - name: Smoke tests
        run: |
          # Wait for deployment
          sleep 30

          # Health check
          curl -f https://example.com/health || exit 1

          # Basic functionality test
          response=$(curl -s https://example.com/api/ping)
          if [ "$response" != '{"status":"ok"}' ]; then
            echo "Smoke test failed"
            exit 1
          fi

      - name: Notify deployment
        if: always()
        run: |
          # Send notification (Slack, Discord, email, etc.)
          echo "Deployment completed"
```

### Environment-Specific Configuration

Use environment variables for configuration:

```vais
U std/env

S Config {
    port: i64,
    db_url: str,
    log_level: str,
    env: str
}

F load_config() -> Config {
    Config {
        port: env::get_int("PORT", 8080),
        db_url: env::get("DATABASE_URL", "postgresql://localhost/myapp"),
        log_level: env::get("LOG_LEVEL", "info"),
        env: env::get("ENV", "development")
    }
}

F main() -> i64 {
    config := load_config()

    I config.env == "production" {
        # Enable strict security
        set_strict_mode(true)
    }

    start_server(config.port)
}
```

---

## Tutorial: REST API to Production

This tutorial walks through building a production-ready REST API from scratch.

### Step 1: Initialize Project

```bash
# Create project structure
mkdir -p todo-api/src
cd todo-api

# Create vais.toml
cat > vais.toml << 'EOF'
[package]
name = "todo-api"
version = "1.0.0"
authors = ["Your Name <you@example.com>"]

[dependencies]
json-parser = "^2.0.0"

[build]
opt_level = 2
EOF
```

### Step 2: Implement REST API

Create `src/main.vais`:

```vais
# REST API for TODO application
U std/http_server
U std/json
U std/file

# Todo item struct
S Todo {
    id: i64,
    title: str,
    completed: bool
}

# Global state (simplified - use proper storage in production)
G todos: [Todo] = []
G next_id: i64 = 1

# GET /todos - List all todos
F handle_list_todos() -> str {
    json::array_to_string(todos)
}

# POST /todos - Create new todo
F handle_create_todo(body: str) -> str {
    title := json::get_string(body, "title")

    todo := Todo {
        id: next_id,
        title: title,
        completed: false
    }

    todos = array_push(todos, todo)
    next_id = next_id + 1

    json::object_to_string(todo)
}

# GET /todos/:id - Get single todo
F handle_get_todo(id: i64) -> str? {
    i := 0
    L i < array_len(todos) {
        todo := todos[i]
        I todo.id == id {
            R Some(json::object_to_string(todo))
        }
        i = i + 1
    }
    None
}

# PUT /todos/:id - Update todo
F handle_update_todo(id: i64, body: str) -> str? {
    i := 0
    L i < array_len(todos) {
        I todos[i].id == id {
            # Update fields
            I json::has_key(body, "title") {
                todos[i].title = json::get_string(body, "title")
            }
            I json::has_key(body, "completed") {
                todos[i].completed = json::get_bool(body, "completed")
            }
            R Some(json::object_to_string(todos[i]))
        }
        i = i + 1
    }
    None
}

# DELETE /todos/:id - Delete todo
F handle_delete_todo(id: i64) -> bool {
    i := 0
    L i < array_len(todos) {
        I todos[i].id == id {
            todos = array_remove(todos, i)
            R true
        }
        i = i + 1
    }
    false
}

# Health check endpoint
F handle_health() -> str {
    "{\"status\":\"ok\",\"todos_count\":" + i64_to_str(array_len(todos)) + "}"
}

F main() -> i64 {
    port := 8080
    app := App::new(port)

    # Configure routes
    app.get("/health", handle_health)
    app.get("/todos", handle_list_todos)
    app.post("/todos", handle_create_todo)
    app.get("/todos/:id", handle_get_todo)
    app.put("/todos/:id", handle_update_todo)
    app.delete("/todos/:id", handle_delete_todo)

    puts("Starting TODO API server on port 8080")
    app.run()
}
```

### Step 3: Add Dockerfile

Create `Dockerfile`:

```dockerfile
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y \
    curl build-essential llvm-17-dev clang-17 libclang-17-dev cmake git \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
RUN git clone https://github.com/vaislang/vais.git && \
    cd vais && \
    cargo build --release --bin vaisc && \
    cp target/release/vaisc /usr/local/bin/vaisc

COPY . /app
WORKDIR /app

RUN vaisc pkg install && \
    vaisc build src/ -o server --optimize 2 --strict-ownership

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 1000 appuser

COPY --from=builder /app/server /usr/local/bin/server

USER appuser
WORKDIR /app

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/server"]
```

### Step 4: Add Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - ENV=production
      - LOG_LEVEL=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3
```

### Step 5: Add CI/CD

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy TODO API

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        run: docker build -t todo-api:${{ github.sha }} .

      - name: Test image
        run: |
          docker run -d -p 8080:8080 --name test-api todo-api:${{ github.sha }}
          sleep 5
          curl -f http://localhost:8080/health || exit 1
          docker stop test-api

      - name: Deploy to production
        run: |
          # Add your deployment commands here
          echo "Deployed successfully"
```

### Step 6: Local Testing

```bash
# Build and run locally
docker-compose up --build

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/todos
curl -X POST http://localhost:8080/todos -d '{"title":"Buy milk"}'
curl http://localhost:8080/todos/1
curl -X PUT http://localhost:8080/todos/1 -d '{"completed":true}'
curl -X DELETE http://localhost:8080/todos/1
```

### Step 7: Production Deployment

```bash
# Tag release
git tag -a v1.0.0 -m "Initial release"
git push origin v1.0.0

# Deploy to cloud provider
# AWS ECS:
# aws ecs update-service --cluster production --service todo-api --force-new-deployment

# Kubernetes:
# kubectl set image deployment/todo-api api=myregistry/todo-api:v1.0.0

# Digital Ocean:
# doctl apps create --spec .do/app.yaml
```

### Step 8: Monitor Production

After deployment, monitor your application:

```bash
# View logs
docker-compose logs -f api

# Check metrics
curl http://localhost:8080/health

# Monitor resource usage
docker stats
```

### Production Best Practices

1. **Use environment variables** for configuration
2. **Enable strict ownership checking** in production builds
3. **Implement structured logging** with log levels
4. **Add health check endpoints** for load balancers
5. **Set up monitoring and alerts** for errors
6. **Use HTTPS** with valid SSL certificates
7. **Rate limit API endpoints** to prevent abuse
8. **Validate all inputs** before processing
9. **Use connection pooling** for databases
10. **Implement graceful shutdown** for zero-downtime deployments

---

## Additional Resources

- [Vais Security Audit Summary](SECURITY_AUDIT_SUMMARY.md) - Security best practices
- [Vais Memory Safety Guide](MEMORY_SAFETY.md) - Ownership and borrowing
- [Vais Standard Library Reference](STDLIB.md) - Built-in modules
- [Vais Package Guidelines](PACKAGE_GUIDELINES.md) - Creating reusable packages
- [Vais Performance Testing Guide](PERFORMANCE_TESTING.md) - Benchmarking and optimization

## Support

For production support and consulting:
- GitHub Issues: https://github.com/vaislang/vais/issues
- Community Forum: https://community.vais-lang.org
- Commercial Support: enterprise@vais-lang.org
