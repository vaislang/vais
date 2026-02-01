# Vais Deployment - Quick Start Guide

Fast-track guide to deploy Vais web properties.

## TL;DR - GitHub Pages (Recommended)

### 1. Enable GitHub Pages
```
https://github.com/vaislang/vais/settings/pages
→ Source: GitHub Actions
→ Custom domain: vais-lang.org
→ Enforce HTTPS: ✓
```

### 2. Configure DNS

Point to GitHub Pages servers:
```
A Records:
185.199.108.153
185.199.109.153
185.199.110.153
185.199.111.153

CNAME:
www → vaislang.github.io
```

### 3. That's It!

Push to main:
```bash
git push origin main
```

Workflows deploy automatically. Check Actions tab.

---

## Deployment Commands Quick Reference

### Website

```bash
cd website

# Development
npm install
npm run dev          # http://localhost:3001

# Build & Test
npm run build
npm run preview

# Manual Deployment (if needed)
# GitHub Pages: Push to main, Actions handles it
# Vercel: vercel --prod
```

### Documentation

```bash
cd docs-site

# Development
mdbook serve         # http://localhost:3000

# Build & Test
mdbook build

# Preview
open book/index.html
```

### Playground

```bash
cd playground

# Development
npm install
npm run dev          # http://localhost:5173

# Build (with WASM)
npm run build
npm run preview
```

---

## Monitoring Deployments

### Check Website Build Status
```
https://github.com/vaislang/vais/actions/workflows/website.yml
```

### Check Docs Build Status
```
https://github.com/vaislang/vais/actions/workflows/docs.yml
```

### Check Playground Build Status
```
https://github.com/vaislang/vais/actions/workflows/playground.yml
```

---

## Common Issues & Fixes

### Site Not Updating

**Problem:** Pushed changes but site hasn't updated
**Solution:**
1. Check Actions tab - is workflow running?
2. Wait 3-5 minutes for deployment
3. Hard refresh browser: `Ctrl+Shift+Del`
4. Check GitHub Pages settings

### Build Failing

**Problem:** Workflow shows red X in Actions
**Solution:**
1. Click workflow run
2. Read error message
3. Test locally:
   - Website: `cd website && npm run build`
   - Docs: `cd docs-site && mdbook build`
   - Playground: `cd playground && npm run build`
4. Fix issue and push again

### HTTPS Not Working

**Problem:** Browser shows certificate error
**Solution:**
1. Wait 10 minutes for certificate issuance
2. Check GitHub Pages settings → "Enforce HTTPS" is enabled
3. Verify DNS is correct
4. Clear browser cache

### Subdomains Not Working

**Problem:** docs.vais-lang.org or play.vais-lang.org showing 404
**Solution:**
1. Verify DNS CNAME records exist
2. Wait up to 48 hours for propagation
3. Test with: `nslookup docs.vais-lang.org`
4. Check GitHub Pages custom domain settings for each

---

## Deployment Architecture

```
You push to main
        ↓
GitHub Actions triggered
        ↓
Build (website, docs, playground)
        ↓
Tests & linting (if configured)
        ↓
Deploy to GitHub Pages / Vercel
        ↓
CDN caches and serves
        ↓
Your visitors see new version
```

---

## Switching to Vercel (Alternative)

If you prefer Vercel:

### 1. Connect Repository
```
https://vercel.com/new
→ Import Vais repository
→ Choose GitHub account
```

### 2. Configure Projects (3 separate projects)

**Website:**
```
Root Directory: website/
Build Command: npm run build
Output Directory: dist/
Custom Domain: vais-lang.org
```

**Docs:**
```
Root Directory: docs-site/
Build Command: mdbook build
Output Directory: book/
Custom Domain: docs.vais-lang.org
```

**Playground:**
```
Root Directory: playground/
Build Command: npm run build
Output Directory: dist/
Custom Domain: play.vais-lang.org
```

### 3. Update DNS
Follow Vercel's DNS instructions in project settings.

### 4. Done!
Vercel auto-deploys on every push.

---

## Environment-Specific Deployments

### Production
- Branch: `main`
- Automatic deployment on push
- Full CI/CD pipeline runs
- All tests must pass

### Staging
- Branch: `staging` (if using Vercel)
- Preview URLs for testing
- No impact on production
- Can be accessed by team for QA

### Local Development
- Branch: any feature branch
- Run locally: `npm run dev`
- Test before pushing to main
- Preview deployment available on Vercel

---

## What Gets Deployed

| Property | Location | Output | Domain |
|----------|----------|--------|--------|
| Website | website/ | dist/ | vais-lang.org |
| Docs | docs-site/ | book/ | docs.vais-lang.org |
| Playground | playground/ | dist/ | play.vais-lang.org |
| Blog | website/blog/ | part of dist/ | vais-lang.org/blog/ |

---

## Performance Metrics

Expected deployment times (after push):

| Service | Build Time | Deploy Time | Total |
|---------|-----------|------------|-------|
| GitHub Pages + Actions | 2-3 min | <1 min | ~3 min |
| Vercel | 1-2 min | <1 min | ~2 min |

---

## Security Best Practices

✓ Always use HTTPS
✓ Enable "Enforce HTTPS" in settings
✓ Use GitHub token (never commit) for manual deployments
✓ Keep Node/Rust versions updated
✓ Review build logs for errors
✓ Test locally before pushing

---

## Need Help?

1. **Check logs:** Actions → Workflow → Run details
2. **Read error:** Usually shows exactly what failed
3. **Test locally:** Reproduce error locally first
4. **See full guide:** Read DEPLOYMENT.md and DEPLOYMENT_STRATEGY.md

---

## Next Deployment Features (Future)

- [ ] Automated performance benchmarks
- [ ] Preview environments for PRs
- [ ] Analytics dashboard
- [ ] Error tracking integration
- [ ] A/B testing support
- [ ] Automatic backups
