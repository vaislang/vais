# Vais Website Deployment

This site publishes the current Vais language and compiler baseline at
`https://vaislang.dev`.

The homepage is bilingual Korean/English and includes a static learning
playground for verified examples. Keep the playground examples aligned with
`tools/vaisc-parity.tsv`.

## Source Of Truth

- Main source: `website/index.html`
- Playground route: `website/playground/index.html`
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

Current live deployment should use `.github/workflows/deploy-website.yml` with
GitHub Pages set to deploy from Actions. On push to `main`, the workflow builds
`website/` and uploads `website/dist` as the Pages artifact.

The `gh-pages` branch may still contain the last manual deployment and preserved
benchmark data, but it is no longer the preferred source once Pages is configured
for Actions.

For manual hosting, deploy the generated `dist/` directory to the host
configured for `vaislang.dev`. The build copies `public/CNAME` into
`dist/CNAME`.

Before deploying, run a stale-public-claim scan over `index.html`, `blog/`,
`ecosystem/`, `vaisx/`, `public/`, and `dist/`.
