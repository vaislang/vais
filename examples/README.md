# Vais Example Corpus

Each `.vais` file with `# expect: N` is part of the value-correctness corpus.

Run:

```bash
bash scripts/test.sh
```

Compiler parity coverage is tracked in `tools/vaisc-parity.tsv` and checked with:

```bash
bash scripts/test-vaisc-parity.sh
```
