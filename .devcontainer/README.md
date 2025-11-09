# DevContainer Configuration

This DevContainer is configured for **Rust development only** and does not include Docker daemon access.

## Scope

The DevContainer environment is designed for:
- Writing and testing Rust code (`cargo build`, `cargo test`, `cargo run`)
- Running the development server locally on port 3000
- Full IDE support with Rust Analyzer, linting, and formatting
- Git operations

## What's NOT Included

- Docker daemon access
- Docker image building
- Docker container management

**Why?** Docker image validation is a deployment concern, not a development concern. It happens in CI/CD pipelines (GitHub Actions), not locally.

## Typical Workflow

```bash
# Inside DevContainer:
cargo test          # Run unit/integration tests
cargo run          # Start development server
cargo clippy       # Lint code
cargo fmt          # Format code
```

## Docker Image Testing

To manually test the Docker image locally (optional):

```bash
# On your host machine (NOT in DevContainer):
./scripts/docker-test.sh
```

This requires Docker and curl installed on the host. See [docs/DOCKER-TESTING.md](../docs/DOCKER-TESTING.md) for details.

## Docker Validation in CI/CD

The GitHub Actions CI/CD pipeline automatically:
1. Builds the Docker image
2. Runs the docker-test.sh script
3. Validates the image is deployable

This happens automatically on every push and pull request.

## Configuration

- **Base Image**: Official Rust DevContainer (bookworm)
- **Extensions**: Rust Analyzer, TOML support, LLDB debugger, Amp
- **Auto-build**: `cargo build` runs on container creation
- **Port Forwarding**: 3000 (development server)
