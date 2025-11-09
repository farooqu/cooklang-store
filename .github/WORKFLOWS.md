# GitHub Actions Workflows

This directory contains the CI/CD pipelines for the Cooklang Store project.

## Available Workflows

### 1. Rust Tests (`rust-tests.yml`)
- **Triggers**: Push to main/develop, Pull Requests
- **Jobs**:
  - Test with stable and nightly toolchains
  - Lint & format checks
  - Security audit for dependency vulnerabilities
- **Duration**: ~1-3 minutes

### 2. Docker Build & Test (`docker-build.yml`)
- **Triggers**: Push to main/develop (when relevant files change), Pull Requests
- **Jobs**:
  - Build Docker image
  - Test Docker image with integration tests
  - Lint Dockerfile with hadolint
- **Duration**: ~3-5 minutes

### 3. Code Coverage (`coverage.yml`)
- **Triggers**: Push to main/develop, Pull Requests
- **Jobs**:
  - Generate coverage report with tarpaulin
  - Upload to Codecov (optional)
  - Comment on PR with coverage %
- **Duration**: ~3-5 minutes

## Local Testing

Test workflows locally before committing using [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or: choco install act-cli  # Windows
# or: https://github.com/nektos/act/releases  # Linux

# Run a specific workflow
act -j test -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:full-latest

# Run all workflows
act -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:full-latest
```

## Monitoring

1. Go to your GitHub repository
2. Click **Actions** tab
3. Select a workflow to see execution history
4. Click a run to view detailed logs

## Cost

**Zero cost** on public repositories. Even on private repositories, you get 2,000 free Actions minutes per month (this project uses ~260/month).

## Documentation

See [../docs/CI-CD.md](../docs/CI-CD.md) for comprehensive documentation, troubleshooting, and extending the pipelines.
