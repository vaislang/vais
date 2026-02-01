# Vais Multi-Property Deployment Strategy

This document outlines the comprehensive deployment strategy for all Vais web properties across multiple hosting platforms.

## Project Structure

```
vais/
├── website/                  # Main landing page & marketing site
│   ├── src/
│   ├── dist/                 # Build output (Vite)
│   ├── blog/                 # Blog posts (integrated into main site)
│   ├── package.json
│   ├── vite.config.js
│   ├── vercel.json          # Vercel configuration
│   └── DEPLOYMENT.md         # Website-specific deployment guide
├── docs-site/               # Official documentation (mdBook)
│   ├── src/
│   ├── book/                 # Build output (mdBook)
│   ├── book.toml
│   ├── build.sh
│   ├── vercel.json          # Vercel configuration
│   └── serve.sh
├── playground/              # Interactive REPL environment
│   ├── src/
│   ├── dist/                 # Build output (Vite)
│   ├── package.json
│   ├── vite.config.js
│   └── vercel.json          # Vercel configuration
└── .github/
    └── workflows/
        ├── website.yml       # Website CI/CD
        ├── docs.yml          # Docs CI/CD
        └── playground.yml    # Playground CI/CD
```

## Deployment Properties

### 1. Main Website

**Domain:** `vais-lang.org`
**Path:** `website/`
**Build Tool:** Vite
**Build Command:** `npm run build`
**Output Directory:** `dist/`

#### Deployment Options

**GitHub Pages:**
1. Enable in repository settings
2. Set custom domain to `vais-lang.org`
3. Automatic HTTPS (Let's Encrypt)
4. Deploy via GitHub Actions (`.github/workflows/website.yml`)

**Vercel:**
1. Connect Vais repository to Vercel
2. Set root directory to `website/`
3. Build command: `npm run build`
4. Output directory: `dist/`
5. Add custom domain `vais-lang.org`

**Deployment Process:**
- Push to `main` branch
- GitHub Actions/Vercel triggers automatically
- Build step: `npm ci && npm run build`
- Deploy `dist/` folder to CDN

### 2. Documentation Site

**Domain:** `docs.vais-lang.org`
**Path:** `docs-site/`
**Build Tool:** mdBook
**Build Command:** `mdbook build`
**Output Directory:** `book/`

#### Deployment Options

**GitHub Pages (Separate Repo):**
1. Create separate docs repository
2. or: Deploy to separate GitHub Pages site
3. Set custom domain to `docs.vais-lang.org`

**GitHub Pages (Same Repo):**
1. Create orphan branch `docs-pages`
2. Push built `docs-site/book/` to this branch
3. Configure GitHub Pages to use `docs-pages` branch
4. Workflow (`.github/workflows/docs.yml`) handles automation

**Vercel:**
1. Deploy as separate project
2. Root directory: `docs-site/`
3. Build command: `mdbook build`
4. Output directory: `book/`
5. Custom domain: `docs.vais-lang.org`

**Important Notes:**
- mdBook requires Rust to build (no Node.js)
- Automatic link checking available via mdbook-linkcheck
- Search functionality built-in to mdBook output
- Edit buttons link to GitHub for community contributions

### 3. Playground (REPL)

**Domain:** `play.vais-lang.org`
**Path:** `playground/`
**Build Tool:** Vite
**Build Command:** `npm run build`
**Output Directory:** `dist/`

#### Special Considerations

- Requires WASM compilation: `npm run wasm:build` (compiles vaisc to WebAssembly)
- Package includes Monaco Editor for syntax highlighting
- Heavy client-side JavaScript application

#### Deployment Options

**GitHub Pages:**
1. Configure as separate Pages site
2. Set custom domain to `play.vais-lang.org`
3. WASM files properly served with correct MIME types
4. Workflow (`.github/workflows/playground.yml`) handles WASM build

**Vercel:**
1. Deploy as separate project
2. Root directory: `playground/`
3. Install command: `npm install`
4. Build command: `npm run build` (includes WASM)
5. Output directory: `dist/`
6. Custom domain: `play.vais-lang.org`
7. Vercel automatically handles WASM MIME types

#### WASM Build Process
```
1. Cargo builds vaisc for wasm32-unknown-unknown target
2. Output WASM binary placed in dist/
3. JavaScript loads and instantiates WASM module
4. Browser runs Vais code in WebAssembly sandbox
```

### 4. Blog

**Domain:** `vais-lang.org/blog`
**Path:** `website/blog/`
**Build Tool:** Part of website (Vite)
**Integration:** Included in main website build

#### Notes
- Blog content served at subdirectory of main site
- Builds as part of website build pipeline
- No separate deployment needed

## DNS Configuration

### For GitHub Pages Hosting

**Primary Domain (vais-lang.org):**
```
Type: A
Name: @
Values:
  185.199.108.153
  185.199.109.153
  185.199.110.153
  185.199.111.153

Type: AAAA
Name: @
Values:
  2606:50c0:8000::153
  2606:50c0:8001::153
  2606:50c0:8002::153
  2606:50c0:8003::153
```

**WWW Subdomain:**
```
Type: CNAME
Name: www
Value: vaislang.github.io
```

**Subdomains (if using GitHub Pages):**
```
Type: CNAME
Name: docs
Value: vaislang.github.io  (if separate repo: docs-repo.github.io)

Type: CNAME
Name: play
Value: vaislang.github.io  (if separate repo: playground-repo.github.io)
```

### For Vercel Hosting

**Primary Domain:**
```
Type: A / CNAME
Name: @
Value: cname.vercel.com
```

**Subdomains:**
```
Type: CNAME
Name: docs
Value: cname.vercel.com

Type: CNAME
Name: play
Value: cname.vercel.com
```

**Note:** Vercel provides exact DNS instructions in project settings after domain addition.

## CI/CD Workflows

### Website Workflow (`.github/workflows/website.yml`)

**Trigger:** Push to `main` with changes to `website/**`

**Steps:**
1. Checkout code
2. Setup Node.js 20 + npm cache
3. Run `npm ci` in website directory
4. Run `npm run build`
5. Upload `dist/` as artifact
6. Deploy to GitHub Pages

**Duration:** ~2-3 minutes
**Failure Notifications:** GitHub Actions notifications

### Documentation Workflow (`.github/workflows/docs.yml`)

**Trigger:** Push to `main` with changes to `docs-site/**`

**Steps:**
1. Checkout code
2. Setup Rust toolchain
3. Cache Rust dependencies
4. Install mdBook and mdbook-linkcheck
5. Run `mdbook build` in docs-site directory
6. Upload `book/` as artifact
7. Deploy to GitHub Pages

**Duration:** ~3-5 minutes (includes Rust compilation)
**Failure Notifications:** GitHub Actions notifications

### Playground Workflow (`.github/workflows/playground.yml`)

**Trigger:** Push to `main` with changes to `playground/**`

**Steps:**
1. Checkout code
2. Setup Node.js 20 + npm cache
3. Run `npm ci` in playground directory
4. Run `npm run build` (includes WASM compilation)
5. Upload `dist/` as artifact
6. Deploy to GitHub Pages

**Duration:** ~2-4 minutes
**Failure Notifications:** GitHub Actions notifications

## Environment Configuration

### GitHub Actions Environment Variables

Create secrets in repository settings for sensitive data:

```
Settings → Secrets and variables → Actions
```

### Vercel Environment Variables

Set in Vercel project dashboard:

```
Project Settings → Environment Variables
```

Common variables:
- `NODE_ENV`: Set to `production` for builds
- `PUBLIC_API_URL`: API endpoint for playground
- `BUILD_PATH`: Custom build directory (if needed)

## Monitoring and Alerts

### GitHub Pages

**Monitor:**
1. Repository → Settings → Pages
2. Last deployment status and time
3. Custom domain validation status

**Logs:**
1. Repository → Actions
2. View workflow run logs
3. Check build output and errors

### Vercel

**Monitor:**
1. Vercel Dashboard → Deployments
2. Real-time deployment status
3. Build logs and duration
4. Performance analytics

**Alerts:**
- Email notifications on deployment failure
- Slack integration available

## Rollback Procedures

### GitHub Pages

**Quick Rollback:**
```bash
git revert <commit-hash>
git push origin main
```

**Manual Rollback:**
1. Actions tab → Failed workflow
2. View successful previous deployment
3. Manually trigger workflow for previous commit

### Vercel

**Easy Rollback:**
1. Vercel Dashboard → Deployments
2. Find previous successful deployment
3. Click "Promote to Production"

## Performance Optimization

### Build Optimization

**Website & Playground:**
- Vite minification enabled by default
- Tree-shaking removes unused code
- CSS/JS bundling and compression

**Documentation:**
- mdBook generates optimized HTML
- Search index built at compile time
- Syntax highlighting pre-generated

### Caching Strategy

**GitHub Pages:**
- Browser caching via Cache-Control headers
- Cloudflare CDN caching (optional)

**Vercel:**
- Automatic edge caching
- Stale-while-revalidate for stability
- 60-second revalidation by default

**Cache Headers (configured in vercel.json):**
```
- /assets/* → 1 year (immutable)
- *.js, *.wasm → 1 week
- HTML files → 1 hour
```

## Testing Before Deployment

### Local Testing

```bash
# Website
cd website
npm install
npm run build
npm run preview
# Open http://localhost:5173

# Docs
cd docs-site
mdbook build
mdbook serve
# Open http://localhost:3000

# Playground
cd playground
npm install
npm run build
npm run preview
# Open http://localhost:5173
```

### Pre-deployment Checklist

- [ ] All dependencies installed
- [ ] Build completes without errors
- [ ] No console warnings or errors
- [ ] Links and navigation work
- [ ] Images and assets load
- [ ] Mobile responsive design functional
- [ ] Forms (if any) working correctly
- [ ] WASM module loads (playground)
- [ ] Search functional (docs)
- [ ] No broken links

## Multi-Environment Deployment

### Development

- Local `npm run dev` on port 3001 (website, playground)
- Local `mdbook serve` for docs
- No deployment needed

### Staging

- Push to `staging` branch
- Deploy to staging URLs or preview links
- Vercel provides preview deployments automatically

### Production

- Push to `main` branch
- Automatic deployment via GitHub Actions
- Full workflow runs with all checks

## Cost Considerations

### GitHub Pages

**Cost:** Free
- Unlimited websites
- 100 GB storage
- Unlimited bandwidth
- No build minutes limit

### Vercel

**Cost:** Free tier includes
- 100 GB bandwidth/month
- Unlimited deployments
- 1000 function invocations/day
- Custom domains

**Recommended for scale:** Pro plan ($20/month) includes faster builds and priority support

## Troubleshooting Guide

### Build Failures

1. **Website build fails:**
   - Check Node.js version: `node --version` (need 18+)
   - Clear cache: `npm ci` (not `npm install`)
   - Check for console errors in build output

2. **Docs build fails:**
   - Ensure Rust installed: `rustc --version`
   - Check mdBook version: `mdbook --version`
   - Verify no markdown syntax errors

3. **Playground build fails:**
   - Check WASM build: `npm run wasm:build`
   - Verify cargo installed
   - Check WASM target installed: `rustup target add wasm32-unknown-unknown`

### Deployment Issues

1. **Site not updating:**
   - Force refresh browser (Ctrl+Shift+Del)
   - Wait 5 minutes for CDN cache
   - Check GitHub Actions workflow status

2. **HTTPS not working:**
   - Wait 10 minutes for certificate issuance
   - GitHub Pages: ensure "Enforce HTTPS" is enabled
   - Check custom domain is correctly configured

3. **404 on subdomains:**
   - Verify DNS records are set correctly
   - Wait 48 hours for DNS propagation
   - Check domain is assigned in platform settings

## Next Steps

1. **Immediate:**
   - Choose GitHub Pages or Vercel
   - Configure custom domains
   - Set up DNS records
   - Enable HTTPS

2. **Short-term:**
   - Monitor deployments
   - Collect performance metrics
   - Gather user feedback

3. **Long-term:**
   - Consider CDN enhancement
   - Implement analytics
   - Set up error tracking
   - Plan SEO optimization

## Additional Resources

- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Vercel Documentation](https://vercel.com/docs)
- [Vite Documentation](https://vitejs.dev/)
- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [DNS Best Practices](https://www.cloudflare.com/learning/dns/dns-best-practices/)
