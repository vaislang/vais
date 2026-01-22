# Deployment Guide

This guide covers various deployment options for the Vais Playground.

## Quick Deploy Options

### 1. Vercel (Recommended)

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new)

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
cd playground
vercel
```

Configuration (`vercel.json`):

```json
{
  "buildCommand": "npm run build",
  "outputDirectory": "dist",
  "devCommand": "npm run dev",
  "framework": "vite"
}
```

### 2. Netlify

[![Deploy to Netlify](https://www.netlify.com/img/deploy/button.svg)](https://app.netlify.com/start)

```bash
# Install Netlify CLI
npm i -g netlify-cli

# Deploy
cd playground
netlify deploy --prod
```

Configuration (`netlify.toml`):

```toml
[build]
  command = "npm run build"
  publish = "dist"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

### 3. GitHub Pages

```bash
# Build
npm run build

# Deploy to gh-pages branch
npm i -g gh-pages
gh-pages -d dist
```

Update `vite.config.js`:

```javascript
export default defineConfig({
  base: '/vais/',  // Replace with your repo name
  // ... rest of config
});
```

### 4. Cloudflare Pages

```bash
# Install Wrangler
npm i -g wrangler

# Deploy
cd playground
wrangler pages publish dist
```

### 5. Static Server

```bash
# Build
npm run build

# Serve with any static server
npx serve dist
# or
python -m http.server 8000 -d dist
# or
php -S localhost:8000 -t dist
```

## Docker Deployment

### Dockerfile

Create `Dockerfile`:

```dockerfile
# Build stage
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy built assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### nginx.conf

```nginx
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # Enable gzip
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript application/wasm;

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|wasm)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
}
```

### Build and Run

```bash
# Build image
docker build -t vais-playground .

# Run container
docker run -d -p 8080:80 vais-playground
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  playground:
    build: .
    ports:
      - "8080:80"
    restart: unless-stopped
    environment:
      - NODE_ENV=production
```

Run:

```bash
docker-compose up -d
```

## Kubernetes Deployment

### Deployment

Create `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vais-playground
  labels:
    app: vais-playground
spec:
  replicas: 3
  selector:
    matchLabels:
      app: vais-playground
  template:
    metadata:
      labels:
        app: vais-playground
    spec:
      containers:
      - name: playground
        image: vais-playground:latest
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Service

Create `k8s/service.yaml`:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: vais-playground
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 80
    protocol: TCP
  selector:
    app: vais-playground
```

### Ingress

Create `k8s/ingress.yaml`:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: vais-playground
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - playground.vais.dev
    secretName: vais-playground-tls
  rules:
  - host: playground.vais.dev
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: vais-playground
            port:
              number: 80
```

### Deploy

```bash
kubectl apply -f k8s/
```

## Performance Optimization

### Build Optimization

```javascript
// vite.config.js
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          monaco: ['monaco-editor']
        }
      }
    },
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true
      }
    }
  }
});
```

### CDN for Dependencies

```html
<!-- In index.html -->
<link rel="modulepreload" href="https://cdn.jsdelivr.net/npm/monaco-editor@0.52.0/min/vs/loader.js">
```

### Preloading

```html
<link rel="preload" as="fetch" href="/wasm/vais_wasm_bg.wasm" crossorigin>
```

### Service Worker

Create `public/sw.js`:

```javascript
const CACHE_NAME = 'vais-playground-v1';
const urlsToCache = [
  '/',
  '/index.html',
  '/src/main.js',
  '/src/styles.css'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => cache.addAll(urlsToCache))
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request)
      .then((response) => response || fetch(event.request))
  );
});
```

Register in `src/main.js`:

```javascript
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js')
    .then(reg => console.log('SW registered', reg))
    .catch(err => console.log('SW error', err));
}
```

## Monitoring

### Analytics

```javascript
// In index.html or main.js
<!-- Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=G-XXXXXXXXXX"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'G-XXXXXXXXXX');
</script>
```

### Error Tracking

```javascript
// Sentry integration
import * as Sentry from "@sentry/browser";

Sentry.init({
  dsn: "YOUR_SENTRY_DSN",
  integrations: [new Sentry.BrowserTracing()],
  tracesSampleRate: 1.0,
});
```

### Performance Monitoring

```javascript
// Log performance metrics
window.addEventListener('load', () => {
  const perfData = window.performance.timing;
  const pageLoadTime = perfData.loadEventEnd - perfData.navigationStart;
  console.log('Page load time:', pageLoadTime);
});
```

## Environment Variables

### Production

Create `.env.production`:

```env
VITE_API_URL=https://api.vais.dev
VITE_WASM_URL=https://cdn.vais.dev/wasm
VITE_ANALYTICS_ID=G-XXXXXXXXXX
```

### Staging

Create `.env.staging`:

```env
VITE_API_URL=https://staging-api.vais.dev
VITE_WASM_URL=https://staging-cdn.vais.dev/wasm
VITE_ANALYTICS_ID=G-YYYYYYYYYY
```

## CI/CD Pipeline

### GitHub Actions

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy Playground

on:
  push:
    branches: [main]
    paths:
      - 'playground/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
          cache-dependency-path: playground/package-lock.json

      - name: Install dependencies
        working-directory: playground
        run: npm ci

      - name: Build
        working-directory: playground
        run: npm run build

      - name: Deploy to Vercel
        uses: amondnet/vercel-action@v20
        with:
          vercel-token: ${{ secrets.VERCEL_TOKEN }}
          vercel-org-id: ${{ secrets.VERCEL_ORG_ID }}
          vercel-project-id: ${{ secrets.VERCEL_PROJECT_ID }}
          working-directory: playground
```

## Security

### Content Security Policy

Add to nginx config or HTML:

```html
<meta http-equiv="Content-Security-Policy" content="
  default-src 'self';
  script-src 'self' 'unsafe-eval' 'unsafe-inline' https://cdn.jsdelivr.net;
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https:;
  font-src 'self' data:;
  connect-src 'self' https://api.vais.dev;
  worker-src 'self' blob:;
  child-src 'self' blob:;
">
```

### HTTPS

Always use HTTPS in production. Most platforms (Vercel, Netlify) provide free SSL.

### Rate Limiting

For API endpoints:

```javascript
// Express.js example
const rateLimit = require('express-rate-limit');

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100 // limit each IP to 100 requests per windowMs
});

app.use('/api/', limiter);
```

## Backup and Recovery

### Database Backups

```bash
# If using a backend database
# Daily backup cron job
0 2 * * * pg_dump playground_db > /backups/playground_$(date +\%Y\%m\%d).sql
```

### Code Versioning

Always tag releases:

```bash
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0
```

## Troubleshooting

### Build Fails

```bash
# Clear cache
rm -rf node_modules package-lock.json
npm install

# Clear Vite cache
rm -rf .vite
```

### WASM Not Loading

Check MIME types:

```nginx
# Add to nginx.conf
types {
    application/wasm wasm;
}
```

### Memory Issues

Increase Node.js memory:

```json
{
  "scripts": {
    "build": "NODE_OPTIONS='--max-old-space-size=4096' vite build"
  }
}
```

## Support

For deployment issues:
- Check the [Vite deployment guide](https://vitejs.dev/guide/static-deploy.html)
- Open an issue on GitHub
- Join the Vais Discord community
