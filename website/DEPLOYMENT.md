# Vais Website Deployment

This site publishes the current Vais language and compiler baseline at
`https://vaislang.dev`.

## Source Of Truth

- Main source: `website/index.html`
- Shared styles: `website/src/styles.css`
- Shared script: `website/src/main.js`
- Built output: `website/dist`
- Domain file: `website/CNAME` and `website/public/CNAME`

The site should describe only the current `.vais` language, `scripts/vaisc`,
self-host status, and verification gates. Package, runtime, or ecosystem pages
should be added only after the relevant repository gates exist.

## Local Build

```bash
cd website
npm install
npm run build
npm run preview
```

## Deploy

The repository includes `.github/workflows/deploy-website.yml` for GitHub Pages.
In the GitHub repository settings, set Pages to deploy from GitHub Actions. On
push to `main` or `master`, the workflow builds `website/` and uploads
`website/dist` as the Pages artifact.

For manual hosting, deploy the generated `dist/` directory to the host
configured for `vaislang.dev`. The build copies `public/CNAME` into
`dist/CNAME`.

Before deploying, run a stale-public-claim scan over `index.html`, `blog/`,
`ecosystem/`, `vaisx/`, `public/`, and `dist/`.
