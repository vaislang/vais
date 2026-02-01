# Vais Deployment Documentation - Complete Guide

This directory contains comprehensive hosting and deployment guides for all Vais web properties.

## Quick Navigation

Start here based on your needs:

### Getting Started
- **First time deploying?** → Read [DEPLOYMENT_QUICKSTART.md](./DEPLOYMENT_QUICKSTART.md)
- **Need DNS setup?** → Read [DNS_CONFIGURATION.md](./DNS_CONFIGURATION.md)

### Detailed Information
- **Website-specific deployment** → Read [website/DEPLOYMENT.md](./website/DEPLOYMENT.md)
- **Overall deployment strategy** → Read [DEPLOYMENT_STRATEGY.md](./DEPLOYMENT_STRATEGY.md)

---

## What's Included

### Documentation Files

| File | Purpose | Audience |
|------|---------|----------|
| [DEPLOYMENT_QUICKSTART.md](./DEPLOYMENT_QUICKSTART.md) | Fast-track deployment setup | Developers, DevOps |
| [DEPLOYMENT_STRATEGY.md](./DEPLOYMENT_STRATEGY.md) | Complete deployment architecture | Tech leads, DevOps |
| [DNS_CONFIGURATION.md](./DNS_CONFIGURATION.md) | Domain and DNS setup | DevOps, Domain admins |
| [website/DEPLOYMENT.md](./website/DEPLOYMENT.md) | Website-specific deployment | Developers |

### Configuration Files

| File | Purpose | Location |
|------|---------|----------|
| [website/vercel.json](./website/vercel.json) | Vercel website config | website/ |
| [docs-site/vercel.json](./docs-site/vercel.json) | Vercel docs config | docs-site/ |
| [playground/vercel.json](./playground/vercel.json) | Vercel playground config | playground/ |

### GitHub Actions Workflows

| File | Trigger | Purpose |
|------|---------|---------|
| [.github/workflows/website.yml](./.github/workflows/website.yml) | website/* changes | Deploy website to GitHub Pages |
| [.github/workflows/docs.yml](./.github/workflows/docs.yml) | docs-site/* changes | Deploy docs to GitHub Pages |
| [.github/workflows/playground.yml](./.github/workflows/playground.yml) | playground/* changes | Deploy playground to GitHub Pages |

---

## Deployment Architecture

### Current Setup

```
┌─────────────────────────────────────────────────────┐
│                    GitHub Repository                │
│                    (vaislang/vais)                   │
├─────────────────────────────────────────────────────┤
│  website/          docs-site/         playground/  │
│  ├── src/          ├── src/           ├── src/     │
│  ├── dist/         ├── book/          ├── dist/    │
│  └── vite.config   ├── book.toml      └── vite.cfg │
└─────────────────────────────────────────────────────┘
          ↓                ↓                    ↓
    GitHub Actions    GitHub Actions     GitHub Actions
         ↓                ↓                    ↓
    npm ci            mdbook            npm ci
    npm build         build             npm build
         ↓                ↓                    ↓
  GitHub Pages     GitHub Pages      GitHub Pages
         ↓                ↓                    ↓
 vais-lang.org  docs.vais-lang.org play.vais-lang.org
```

### Supported Platforms

- **Recommended:** GitHub Pages + GitHub Actions
- **Alternative:** Vercel (all properties)
- **Hybrid:** GitHub Pages for website, Vercel for docs/playground

---

## Five-Minute Setup Guide

### For GitHub Pages (Recommended)

1. **Enable GitHub Pages**
   ```
   Settings → Pages
   → Source: GitHub Actions
   → Custom domain: vais-lang.org
   → Save
   ```

2. **Configure DNS**
   ```
   Add to your registrar:
   - A records: GitHub Pages IPs (see DNS_CONFIGURATION.md)
   - CNAME (www): vaislang.github.io
   ```

3. **Done!**
   - Push to main: workflows deploy automatically
   - Check Actions tab for status
   - Wait 3-5 minutes for live deployment

### For Vercel (Alternative)

1. **Create Projects**
   ```
   Dashboard → Add New → Project
   → Select Vais repository
   → Configure (3 times for website, docs, playground)
   ```

2. **Set Domains**
   - website: vais-lang.org
   - docs: docs.vais-lang.org
   - playground: play.vais-lang.org

3. **Update DNS**
   - Follow Vercel's DNS instructions
   - CNAME to cname.vercel.com

4. **Done!**
   - Auto-deploys on every push
   - Preview URLs available for PRs

---

## File Structure Reference

```
vais/
├── website/
│   ├── DEPLOYMENT.md                    # Website deployment guide
│   ├── vercel.json                      # Vercel configuration
│   ├── package.json                     # Node.js dependencies
│   ├── vite.config.js                   # Vite build config
│   ├── src/                             # Source files
│   ├── dist/                            # Built website
│   └── blog/                            # Blog content
│
├── docs-site/
│   ├── vercel.json                      # Vercel configuration
│   ├── book.toml                        # mdBook config
│   ├── build.sh                         # Build script
│   ├── src/                             # Documentation source
│   └── book/                            # Built documentation
│
├── playground/
│   ├── vercel.json                      # Vercel configuration
│   ├── package.json                     # Node.js dependencies
│   ├── vite.config.js                   # Vite build config
│   ├── src/                             # Source files
│   └── dist/                            # Built playground
│
├── .github/workflows/
│   ├── website.yml                      # Website CI/CD
│   ├── docs.yml                         # Docs CI/CD
│   └── playground.yml                   # Playground CI/CD
│
├── DEPLOYMENT_README.md                 # This file
├── DEPLOYMENT_QUICKSTART.md             # Quick start guide
├── DEPLOYMENT_STRATEGY.md               # Detailed strategy
└── DNS_CONFIGURATION.md                 # DNS setup guide
```

---

## Deployment Properties

### Main Website (vais-lang.org)

- **Source:** website/
- **Build Tool:** Vite
- **Build Command:** npm run build
- **Output:** dist/
- **Deploy Method:** GitHub Pages or Vercel

**Local Testing:**
```bash
cd website
npm install
npm run dev          # http://localhost:3001
npm run build
npm run preview
```

### Documentation (docs.vais-lang.org)

- **Source:** docs-site/
- **Build Tool:** mdBook
- **Build Command:** mdbook build
- **Output:** book/
- **Deploy Method:** GitHub Pages or Vercel

**Local Testing:**
```bash
cd docs-site
mdbook serve         # http://localhost:3000
mdbook build
```

### Playground (play.vais-lang.org)

- **Source:** playground/
- **Build Tool:** Vite
- **Build Command:** npm run build (includes WASM)
- **Output:** dist/
- **Deploy Method:** GitHub Pages or Vercel

**Local Testing:**
```bash
cd playground
npm install
npm run dev          # http://localhost:5173
npm run build
npm run preview
```

### Blog (vais-lang.org/blog)

- **Source:** website/blog/
- **Build Tool:** Part of website
- **Included in:** Main website build

---

## Monitoring & Maintenance

### Check Deployment Status

**GitHub Pages:**
```
https://github.com/vaislang/vais/actions
```

**Vercel:**
```
https://vercel.com/dashboard
```

### Monitor Build Logs

**Website Build:**
```
Actions → Deploy Website → Latest Run
```

**Docs Build:**
```
Actions → Deploy Documentation → Latest Run
```

**Playground Build:**
```
Actions → Deploy Playground → Latest Run
```

### Test Deployments

```bash
# Check website
curl -I https://vais-lang.org

# Check docs
curl -I https://docs.vais-lang.org

# Check playground
curl -I https://play.vais-lang.org

# Verify DNS
nslookup vais-lang.org
nslookup docs.vais-lang.org
nslookup play.vais-lang.org
```

---

## Common Tasks

### Deploy Website

```bash
cd website
npm run build
# Push to main - GitHub Actions handles deployment
git push origin main
```

### Update Documentation

```bash
cd docs-site
# Edit markdown files in src/
mdbook build  # Test locally
# Push to main - GitHub Actions handles deployment
git push origin main
```

### Deploy Playground Update

```bash
cd playground
npm run build  # Tests build including WASM
# Push to main - GitHub Actions handles deployment
git push origin main
```

### Roll Back to Previous Version

**GitHub Pages:**
```bash
# Find previous commit
git log --oneline

# Revert to specific commit
git revert <commit-hash>
git push origin main
```

**Vercel:**
```
Dashboard → Deployments → Find previous successful deployment
→ Click "Promote to Production"
```

### Check Build Status

```bash
# View GitHub Actions status
https://github.com/vaislang/vais/actions

# Or via command line
gh run list --branch main --limit 10
```

---

## Troubleshooting

### Build Failing

1. Check Actions tab for error
2. Common issues:
   - Node version mismatch: ensure Node 18+
   - Missing dependencies: run `npm ci` locally
   - Build errors: test `npm run build` locally

3. Fix and retry:
```bash
# Test locally
cd website
npm ci
npm run build

# If it works locally, push to main
git push origin main
```

### Site Not Updating

1. Check Actions tab - is workflow running?
2. Wait 3-5 minutes for deployment
3. Hard refresh browser (Ctrl+Shift+Del)
4. Check GitHub Pages settings for correct domain

### HTTPS Not Working

1. Wait 10 minutes for certificate (Let's Encrypt)
2. Ensure "Enforce HTTPS" is enabled in GitHub Pages
3. Verify custom domain is set correctly
4. Check DNS records are correct

### Subdomain Not Working

1. Verify DNS CNAME record exists
2. Wait up to 48 hours for propagation
3. Test with: `nslookup docs.vais-lang.org`
4. Check GitHub Pages settings

**See [DNS_CONFIGURATION.md](./DNS_CONFIGURATION.md) for detailed troubleshooting.**

---

## Performance Targets

### Build Times

| Property | Expected | Notes |
|----------|----------|-------|
| Website | 2-3 min | Vite + Node.js |
| Docs | 3-5 min | Includes Rust compilation |
| Playground | 2-4 min | Includes WASM build |

### Deployment Times

| Platform | Expected |
|----------|----------|
| GitHub Pages | <1 min |
| Vercel | <1 min |

### Page Load Times

| Site | Target | Current |
|------|--------|---------|
| vais-lang.org | <1s | CDN cached |
| docs.vais-lang.org | <2s | CDN cached |
| play.vais-lang.org | <2s | Heavy JS app |

---

## Security Checklist

- [x] HTTPS enabled on all domains
- [x] Custom domains configured
- [x] GitHub token not committed
- [x] No secrets in build logs
- [x] Build cache clean
- [x] Dependencies up to date
- [ ] DNSSEC enabled (optional)
- [ ] Security headers configured (ready in vercel.json)

---

## Next Steps

1. **Immediate**
   - Choose hosting platform (GitHub Pages recommended)
   - Configure DNS records (see [DNS_CONFIGURATION.md](./DNS_CONFIGURATION.md))
   - Enable HTTPS in platform settings
   - Test with `nslookup` command

2. **Short-term**
   - Monitor first few deployments
   - Verify all subdomains working
   - Collect performance metrics
   - Set up alerts if needed

3. **Long-term**
   - Monitor build times
   - Optimize bundle sizes
   - Consider edge caching
   - Plan future scaling

---

## Getting Help

### Documentation

- Website setup: [website/DEPLOYMENT.md](./website/DEPLOYMENT.md)
- Overall strategy: [DEPLOYMENT_STRATEGY.md](./DEPLOYMENT_STRATEGY.md)
- DNS issues: [DNS_CONFIGURATION.md](./DNS_CONFIGURATION.md)
- Quick help: [DEPLOYMENT_QUICKSTART.md](./DEPLOYMENT_QUICKSTART.md)

### GitHub Issues

Report issues at: https://github.com/vaislang/vais/issues

### Community

- [Vais GitHub Discussions](https://github.com/vaislang/vais/discussions)
- [GitHub Pages Support](https://github.com/contact/github-pages)
- [Vercel Support](https://vercel.com/support)

---

## Additional Resources

- [GitHub Pages Docs](https://docs.github.com/en/pages)
- [Vercel Docs](https://vercel.com/docs)
- [Vite Docs](https://vitejs.dev/)
- [mdBook Docs](https://rust-lang.github.io/mdBook/)
- [GitHub Actions](https://docs.github.com/en/actions)

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-30 | Initial comprehensive deployment guide |

---

## Summary

You now have everything needed to:
- Deploy the Vais website to GitHub Pages or Vercel
- Configure custom domains and DNS
- Set up automated CI/CD pipelines
- Monitor and troubleshoot deployments
- Scale to multiple properties (website, docs, playground)

**Start with [DEPLOYMENT_QUICKSTART.md](./DEPLOYMENT_QUICKSTART.md) for immediate setup, then refer to specific guides as needed.**
