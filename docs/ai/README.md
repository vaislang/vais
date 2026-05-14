# AI Documentation Sync

This directory contains AI-facing Vais onboarding docs.

Authoritative edit point:

- `docs/ai/feature-registry.json`

Generated docs:

- `docs/ai/LLM_LANGUAGE_CARD.md`
- `docs/ai/AI_DEVELOPER_GUIDE.md`
- `docs/ai/REFERENCE_APP_CONTRACT.md`

Local workflow:

```bash
node scripts/generate-ai-docs.mjs
node scripts/check-ai-docs-sync.mjs
```

When language, compiler, playground, DB, server, or web behavior changes, update
the feature registry first and regenerate the docs. CI runs
`scripts/check-ai-docs-sync.mjs` so generated files cannot drift from the
registry.
