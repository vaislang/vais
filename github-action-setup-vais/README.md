# Setup Vais GitHub Action

A GitHub Action to install the Vais programming language compiler on your CI/CD pipeline.

## Quick Start

Add this to your workflow file (`.github/workflows/your-workflow.yml`):

```yaml
- uses: vais-lang/setup-vais@v1
  with:
    version: latest
```

This will:
1. Detect your OS (Linux, macOS, Windows) and architecture (x86_64, aarch64)
2. Download the appropriate Vais compiler binary from GitHub Releases
3. Extract and add the binary to your PATH
4. Verify the installation by running `vaisc --version`

## Full Example with Matrix Build

```yaml
name: Test Vais Compiler

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        vais-version: [latest, "0.1.0"]

    steps:
      - uses: actions/checkout@v4

      - name: Setup Vais
        uses: vais-lang/setup-vais@v1
        with:
          version: ${{ matrix.vais-version }}
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Compile Vais project
        run: vaisc build

      - name: Run tests
        run: vaisc test
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `version` | Version of Vais to install | No | `latest` |
| `token` | GitHub token for API requests (helps with rate limits) | No | (empty) |

### Version Input

- `latest` - Downloads the most recent release from GitHub
- `0.1.0` - Specific version number (without the `v` prefix)
- `v0.1.0` - Version with `v` prefix (both formats work)

### Token Input

Providing a GitHub token is optional but recommended for CI/CD environments to avoid GitHub API rate limits:

```yaml
- uses: vais-lang/setup-vais@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

## Supported Platforms

The action automatically detects and installs the correct binary for your platform:

| OS | Architectures |
|----|---------------|
| Linux | x86_64, aarch64 |
| macOS | x86_64, aarch64 |
| Windows | x86_64 |

Binaries are downloaded from the [vais-lang/vais](https://github.com/vais-lang/vais) GitHub repository releases.

## Environment Variables

After the action completes, `vaisc` is available in your workflow's PATH and can be used in subsequent steps:

```yaml
steps:
  - uses: vais-lang/setup-vais@v1

  - run: vaisc --version
  - run: vaisc build src/main.vais
```

## Troubleshooting

### Version not found

If you specify a version that doesn't exist:

```
Failed to download vaisc-0.2.0-x86_64-unknown-linux-gnu.tar.gz
```

Check that the version is available in the [releases page](https://github.com/vais-lang/vais/releases).

### Binary not found for platform

If your platform/architecture combination is not supported:

```
Unsupported OS or architecture: [your platform]
```

Please open an issue on the [Vais repository](https://github.com/vais-lang/vais).

### Rate limit exceeded

If you see GitHub API rate limit errors, provide a token:

```yaml
- uses: vais-lang/setup-vais@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

## License

This action is part of the Vais project and is released under the same license.

## Contributing

To contribute improvements to this action, please submit a pull request to the [vais-lang/vais](https://github.com/vais-lang/vais) repository.
