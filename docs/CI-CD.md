# CI/CD Pipeline

CookLang Backend uses GitHub Actions for continuous integration and continuous deployment. All workflows run on GitHub-hosted runners (Ubuntu latest) at no additional cost.

## Workflow Overview

### 1. Rust Tests (`rust-tests.yml`)

**Trigger**: Push to `main` or `develop`, Pull Requests

**Jobs**:
- **test**: Run Rust test suite with both stable and nightly toolchains
  - Runs on matrix: `[stable, nightly]`
  - Caches cargo registry, git index, and build artifacts
  - Executes `cargo test --verbose`
  - Tests run with single-threaded mode to catch race conditions
  
- **lint**: Code quality checks
  - Format verification: `cargo fmt -- --check`
  - Linter: `cargo clippy --all-targets -- -D warnings`
  - Prevents style issues and common Rust pitfalls

- **security**: Dependency vulnerability scanning
  - Uses `rustsec/audit-check-action` to detect known CVEs
  - Fails the build if vulnerabilities are found

**Duration**: ~2-3 minutes (first run) to ~1 minute (cached)

**Cost**: Free (GitHub-hosted runner)

### 2. Docker Build & Test (`docker-build.yml`)

**Trigger**: 
- Push to `main` or `develop` (when relevant files change)
- Pull Requests (when relevant files change)
- Relevant files: `src/**`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `scripts/docker-test.sh`

**Jobs**:
- **build**: Build Docker image using Docker Buildx
  - Uses GitHub Actions cache for layer caching
  - Builds without pushing to registry
  - Outputs image for downstream jobs

- **test**: Integration testing of Docker image
  - Depends on `build` job
  - Loads built image and runs `scripts/docker-test.sh`
  - Tests core API endpoints without requiring Rust
  - Verifies containerized deployment works correctly

- **lint**: Dockerfile validation
  - Uses `hadolint` to check Dockerfile best practices
  - Detects missing HEALTHCHECK, unoptimized layers, etc.
  - Warns on style issues, fails on critical problems

**Duration**: ~3-5 minutes

**Cost**: Free (GitHub-hosted runner, Docker included)

### 3. Code Coverage (`coverage.yml`)

**Trigger**: Push to `main` or `develop`, Pull Requests

**Jobs**:
- **coverage**: Generate code coverage report
  - Installs `cargo-tarpaulin` for coverage analysis
  - Generates Cobertura XML report
  - Uploads to Codecov (optional, requires `CODECOV_TOKEN` secret)
  - Comments on PRs with coverage percentage
  - Maintains >80% coverage requirement

**Duration**: ~3-5 minutes

**Cost**: Free (GitHub-hosted runner)

## Running Workflows Locally

Test workflows before committing using [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or download from https://github.com/nektos/act

# Run a specific workflow
act -j test -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:full-latest

# Run all workflows
act -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:full-latest
```

## Monitoring Workflows

### GitHub UI

1. Go to your repository on GitHub
2. Click **Actions** tab
3. Select a workflow to see execution details
4. Click a run to view logs

### Workflow Status Badge

Add to your README to display pipeline status:

```markdown
[![Rust Tests](https://github.com/YOUR_USERNAME/cooklang-backend/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/YOUR_USERNAME/cooklang-backend/actions/workflows/rust-tests.yml)
[![Docker Build](https://github.com/YOUR_USERNAME/cooklang-backend/actions/workflows/docker-build.yml/badge.svg)](https://github.com/YOUR_USERNAME/cooklang-backend/actions/workflows/docker-build.yml)
```

## Caching

Workflows use GitHub's cache action to speed up builds:

### Cargo Cache Keys

- `~/.cargo/registry` - Crates index (keyed by `Cargo.lock`)
- `~/.cargo/git` - Git dependencies (keyed by `Cargo.lock`)
- `target/` - Build artifacts (keyed by `Cargo.lock`)

**Cache hit saves ~90 seconds per test run**

### Docker Cache

- Layer caching via Docker Buildx (type=gha)
- Significantly speeds up Docker image builds
- Automatically managed by GitHub Actions

## Secrets & Configuration

### Required Secrets

None required for basic CI/CD. Optional:

- **CODECOV_TOKEN** (optional)
  - For uploading coverage to Codecov
  - Get from https://codecov.io
  - Add in GitHub Settings > Secrets > New repository secret

### Environment Variables

Currently set in workflows:
- `RUST_LOG=info` (implicitly from tests)
- Can be added via workflow environment blocks

## Troubleshooting

### Rust Tests Failing

1. Check logs in GitHub Actions UI
2. Common issues:
   - Dependency conflicts: `cargo update`
   - Formatting: `cargo fmt`
   - Clippy warnings: `cargo clippy --fix`
3. Run locally: `cargo test`

### Docker Tests Failing

1. Check `docker-test.sh` output in logs
2. Common issues:
   - Port 3000 in use locally: use different port
   - Git configuration missing: `git config --global user.email "test@example.com"`
3. Run locally: `./scripts/docker-test.sh`

### Coverage Not Uploading

1. Verify `CODECOV_TOKEN` is set in repository secrets
2. Check Codecov account is linked
3. Coverage report is still generated locally even if upload fails

### Workflow Not Triggering

1. Verify branch filter matches your branch
2. Verify file path filters (if any) include changed files
3. Check branch protection rules don't require additional approvals

## GitHub Actions Cost

**Zero cost**: All workflows use GitHub-hosted runners on public repositories.

Even on private repositories, you get 2,000 free Actions minutes per month, which is more than enough for this project:
- Rust tests: ~60 minutes/month
- Docker tests: ~100 minutes/month
- Coverage: ~100 minutes/month
- **Total**: ~260 minutes/month well under limit

## Extending the Pipeline

### Adding More Workflows

Create new files in `.github/workflows/`:

```yaml
name: My New Workflow

on:
  push:
    branches: [main]
  pull_request:

jobs:
  my-job:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # Your workflow here
```

### Common Additions

**Deployment Workflow** (when ready):
```yaml
- name: Deploy to Production
  if: github.ref == 'refs/heads/main'
  run: |
    # Your deployment script here
```

**Release Workflow**:
```yaml
on:
  push:
    tags:
      - 'v*'
```

**Schedule Workflow** (run on schedule):
```yaml
on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight UTC
```

## Best Practices

1. **Keep workflows fast**: Use caching, avoid unnecessary steps
2. **Fail fast**: Run quick checks (format, lint) before slow tests
3. **Reuse actions**: Use community actions for common tasks
4. **Document changes**: Update this file when adding/modifying workflows
5. **Test locally**: Use `act` before committing workflow changes
6. **Semantic versioning**: Pin action versions (`@v4` not `@latest`)

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Toolchain Action](https://github.com/dtolnay/rust-toolchain)
- [Docker Build Action](https://github.com/docker/build-push-action)
- [Codecov Action](https://github.com/codecov/codecov-action)
- [act - Run GitHub Actions locally](https://github.com/nektos/act)
