# Docker Testing Guide

## Overview

The `scripts/docker-test.sh` script provides a standalone way to test the CookLang Backend Docker image without requiring Rust to be installed. This is ideal for CI/CD pipelines, deployment testing, and environments where only Docker is available.

## Prerequisites

- Docker (any recent version)
- curl (for HTTP tests)
- jq (for JSON output formatting, optional)

## Running Tests

```bash
./scripts/docker-test.sh
```

The script will:
1. Create an isolated test recipe repository in `/tmp/cooklang-test-recipes-$$`
2. Initialize a git repository with sample recipes
3. Build the Docker image
4. Start a container with the test recipes mounted
5. Wait for the API to be ready
6. Run HTTP tests against the API endpoints
7. Clean up all resources (container, images, test repos)

## Test Coverage

The script tests the following endpoints:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/api/v1/status` | GET | Server status and statistics |
| `/api/v1/categories` | GET | List all recipe categories |
| `/api/v1/recipes` | GET | List all recipes with pagination |
| `/api/v1/recipes/search` | GET | Search recipes by name |
| `/api/v1/recipes` | POST | Create a new recipe |

## Test Repository Structure

The test script creates a minimal git repository with the following structure:

```
/tmp/cooklang-test-recipes-XXXX/
├── .git/                          # Git repository
└── recipes/
    └── desserts/
        └── chocolate-cake.cook    # Sample chocolate cake recipe
```

The repository is initialized with:
- Git user: `test@example.com` / `Test User`
- Initial commit: Sample recipes

## Test Data

### Sample Recipe

The test includes a sample chocolate cake recipe to demonstrate:
- Basic ingredient parsing (`@ingredient{quantity%unit}`)
- Cookware references (`#cookware{}`)
- Timer specifications (`~duration{}`)
- Recipe metadata (servings)

## Cleanup

All resources are automatically cleaned up after tests complete:
- Docker container (stopped and removed)
- Test recipe repository (deleted from `/tmp`)

The script uses a process ID (`$$`) in directory names to ensure isolation when running multiple tests concurrently.

## Exit Codes

- `0`: All tests passed
- `1`: One or more tests failed, or API failed to start

## Environment Variables

The test configures the following environment variables in the container (uses defaults):

- `RUST_LOG=info` - Logging level (default)
- `DATA_DIR=/recipes` - Recipe storage path (default)
- `STORAGE_TYPE=disk` - Storage backend (default)

See [Dockerfile](../Dockerfile) for default configuration.

## Troubleshooting

### Port 3000 Already in Use

If you see "port is already allocated" error:

```bash
# Stop any existing containers
docker stop cooklang-store-test-* 2>/dev/null || true
docker rm cooklang-store-test-* 2>/dev/null || true

# Try again
./scripts/docker-test.sh
```

### API Not Responding

If tests fail because the API didn't start:

1. Check Docker is running: `docker ps`
2. Check image builds: `docker build -t cooklang-store-test .`
3. Run container manually: `docker run -p 3000:3000 cooklang-store-test`
4. Check logs: `docker logs cooklang-store-test`

### Git Configuration Missing

If you see git errors during test repo creation:

```bash
# Ensure git is configured globally
git config --global user.email "test@example.com"
git config --global user.name "Test User"
```

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
name: Docker Tests

on: [push, pull_request]

jobs:
  docker-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Docker tests
        run: ./scripts/docker-test.sh
```

## Notes

- Tests use HTTP (not HTTPS) - suitable for local testing only
- Test containers are isolated with unique names per run
- The script logs detailed output with color coding (green = pass, red = fail, yellow = info)
