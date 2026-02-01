# Vais Website - Hosting and Deployment Guide

This guide covers deployment options and strategies for the Vais website and related properties.

## Overview

The Vais project consists of multiple web properties:
- **Main website** (website/) → `vais-lang.org`
- **Documentation** (docs-site/) → `docs.vais-lang.org`
- **Playground** (playground/) → `play.vais-lang.org`
- **Blog** (website/blog/) → `vais-lang.org/blog/`

## Deployment Options

### Option 1: GitHub Pages (Recommended)

GitHub Pages is the simplest and most cost-effective option for static sites. It integrates directly with your repository and provides automatic SSL/HTTPS.

#### Setup for Main Website

1. **Enable GitHub Pages**
   - Go to https://github.com/vaislang/vais
   - Navigate to Settings → Pages
   - Under "Build and deployment":
     - Source: Deploy from a branch
     - Branch: `main`
     - Folder: `/website` (if using subdirectory) or `/website/dist` (if using dist folder)
     - Or use GitHub Actions (recommended - see below)

2. **Custom Domain Configuration**

   Choose one of the following:

   **Option A: Using vais-lang.org (Recommended)**
   ```
   In GitHub Pages settings:
   - Custom domain: vais-lang.org
   - Enforce HTTPS: ✓ (checked)
   ```

   **Option B: Using vaislang.dev**
   ```
   In GitHub Pages settings:
   - Custom domain: vaislang.dev
   - Enforce HTTPS: ✓ (checked)
   ```

3. **DNS Configuration for vais-lang.org**

   If you own the domain, configure DNS records:

   **For apex domain (vais-lang.org):**
   ```
   Type: A
   Name: @
   Value: 185.199.108.153

   Type: A
   Name: @
   Value: 185.199.109.153

   Type: A
   Name: @
   Value: 185.199.110.153

   Type: A
   Name: @
   Value: 185.199.111.153
   ```

   **For www subdomain (www.vais-lang.org):**
   ```
   Type: CNAME
   Name: www
   Value: vaislang.github.io
   ```

4. **SSL/HTTPS**

   GitHub Pages automatically issues an SSL certificate via Let's Encrypt. No additional configuration needed.
   - Enable "Enforce HTTPS" in GitHub Pages settings
   - Wait 5-10 minutes for the certificate to be issued

#### Setup for Documentation (docs.vais-lang.org)

Create a separate docs-site repository or use a subdomain:

1. Build documentation site
2. Deploy to GitHub Pages on `docs-site` branch
3. Set custom domain to `docs.vais-lang.org`

Alternatively, if docs-site is in the same repo:
```
In GitHub Pages settings:
- Source: GitHub Actions
- Deploy built docs-site/ to docs subdirectory
```

#### Setup for Playground (play.vais-lang.org)

Similar to documentation:
1. Build playground
2. Deploy via GitHub Pages
3. Set custom domain to `play.vais-lang.org`

#### Automated Deployment with GitHub Actions

The recommended approach is using GitHub Actions for building and deploying.

See `.github/workflows/website.yml` for the automated workflow that:
- Triggers on push to main (website/* changes)
- Runs `npm install && npm run build`
- Deploys to GitHub Pages branch automatically

**Advantages:**
- Build happens on GitHub (reproducible environment)
- Automatic deployment on every commit
- Easy to add additional build steps or validations

### Option 2: Vercel Deployment

Vercel is an excellent alternative with fast edge deployments and built-in analytics.

#### Setup for Main Website

1. **Install Vercel CLI**
   ```bash
   npm i -g vercel
   ```

2. **Create vercel.json**
   ```json
   {
     "buildCommand": "npm run build",
     "outputDirectory": "dist",
     "public": true,
     "env": [],
     "regions": ["iad1"]
   }
   ```

3. **Deploy**
   ```bash
   cd website
   vercel --prod
   ```

4. **Configure Custom Domain**
   - Go to Vercel dashboard → Project Settings → Domains
   - Add `vais-lang.org`
   - Follow DNS instructions (usually CNAME to vercel.com)

5. **Environment Variables**
   - Vercel Dashboard → Settings → Environment Variables
   - Add any required environment variables for the build

#### Setup for Documentation

1. Create separate Vercel project for docs-site
2. Configure custom domain as `docs.vais-lang.org`
3. Point DNS CNAME to Vercel

#### Setup for Playground

1. Create separate Vercel project for playground
2. Configure custom domain as `play.vais-lang.org`
3. Point DNS CNAME to Vercel

#### GitHub Integration (Recommended)

Instead of manual deploys, integrate with GitHub:

1. Go to Vercel → Add New → Project
2. Select the Vais repository
3. Configure build settings:
   - Framework: Other (or manual)
   - Build Command: `npm run build`
   - Output Directory: `website/dist`
4. Vercel automatically deploys on every push

## Unified Deployment Strategy

### Directory Structure
```
vais/
├── website/                  # Main website
│   ├── src/
│   ├── dist/                 # Built output
│   ├── package.json
│   ├── vite.config.js
│   └── blog/                 # Blog content
├── docs-site/               # Documentation
│   ├── src/
│   ├── dist/
│   └── package.json
├── playground/              # REPL Playground
│   ├── src/
│   ├── dist/
│   └── package.json
└── .github/workflows/
    └── website.yml          # Automated deployment
```

### Deployment Targets
| Property | Domain | Source | Build | Deploy |
|----------|--------|--------|-------|--------|
| Website | vais-lang.org | website/ | Vite | GitHub Pages or Vercel |
| Docs | docs.vais-lang.org | docs-site/ | Docs build | GitHub Pages or Vercel |
| Playground | play.vais-lang.org | playground/ | Vite | GitHub Pages or Vercel |
| Blog | vais-lang.org/blog | website/blog/ | Part of website | GitHub Pages or Vercel |

### Multi-Property Workflow

If deploying all properties to GitHub Pages:

1. **Website builds to:** `dist/` at root (served at vais-lang.org)
2. **Docs builds to:** `dist/docs/` (served at docs.vais-lang.org via separate repo)
3. **Playground builds to:** `dist/play/` (served at play.vais-lang.org via separate repo)
4. **Blog included in:** website build

Or use separate repositories for each property for better isolation.

## GitHub Actions Workflow

The workflow file (`.github/workflows/website.yml`) handles automatic deployment:

```yaml
# Triggers on push to main when website files change
# Runs npm install && npm run build
# Deploys dist/ to GitHub Pages gh-pages branch
```

To use:
1. File is already created at `.github/workflows/website.yml`
2. Ensure GitHub Pages is set to deploy from `gh-pages` branch
3. Every push to main updates the website

## Local Development

### Build and Test Locally

```bash
cd website

# Install dependencies
npm install

# Development server (http://localhost:3001)
npm run dev

# Production build
npm run build

# Preview built site
npm run preview
```

### Testing Before Deployment

```bash
# Build locally
npm run build

# Verify dist/ folder contains expected files
ls -la dist/

# Preview the build
npm run preview
```

## DNS Configuration

### If Using GitHub Pages

**vais-lang.org setup:**
1. Add A records pointing to GitHub Pages IPs (see GitHub Pages section)
2. Add CNAME record for www.vais-lang.org → vaislang.github.io

**Subdomains (docs, play):**
1. Add CNAME records for docs and play subdomains
2. Point to respective GitHub Pages or Vercel endpoints

### If Using Vercel

**vais-lang.org setup:**
1. Add CNAME record: vais-lang.org → cname.vercel.com (or as instructed)

**Subdomains:**
1. Add separate CNAME records for each subdomain

## Monitoring and Maintenance

### GitHub Pages
- Monitor deployments: Repository → Actions tab
- Check deployment status: Settings → Pages
- View build logs: Actions → workflow runs

### Vercel
- Dashboard shows deployment history
- Real-time analytics available
- Edge function logs in dashboard

## Troubleshooting

### Site Not Updating
1. Check GitHub Actions workflow status (Actions tab)
2. Verify build logs for errors
3. Ensure correct branch is deployed
4. Clear browser cache (Ctrl+Shift+Del)
5. Wait 5 minutes for DNS propagation

### Custom Domain Not Working
1. Verify DNS records are correct
2. Wait up to 48 hours for DNS propagation
3. Check GitHub Pages settings for domain configuration
4. Ensure HTTPS is enforced

### Build Failures
1. Check workflow logs in Actions tab
2. Run `npm run build` locally to reproduce
3. Verify all dependencies are installed
4. Check for missing environment variables

### HTTPS Not Working
1. GitHub Pages: Enable "Enforce HTTPS" and wait 10 minutes
2. Vercel: Should be automatic, check certificate status
3. Ensure domain DNS is properly configured

## Production Checklist

Before deploying to production:

- [ ] Build passes locally: `npm run build`
- [ ] No console errors in preview: `npm run preview`
- [ ] All assets load correctly
- [ ] Links and navigation work
- [ ] Mobile responsive design working
- [ ] DNS records are correct
- [ ] HTTPS is enabled
- [ ] GitHub Pages/Vercel settings are correct
- [ ] GitHub Actions workflow is enabled and passing
- [ ] Custom domain is configured

## Rollback Procedures

### GitHub Pages
1. Go to Actions tab
2. Find previous successful deployment
3. Manual rollback via git revert:
   ```bash
   git revert <commit-hash>
   git push origin main
   ```

### Vercel
1. Go to Deployments tab
2. Click "Promote to Production" on previous successful deployment

## Additional Resources

- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Vercel Documentation](https://vercel.com/docs)
- [Vite Build Guide](https://vitejs.dev/guide/build.html)
- [DNS Records Explained](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-site/managing-a-custom-domain-for-your-github-pages-site)
