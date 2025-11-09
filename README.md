# CookLang Recipe Backend

A backend service for managing, storing, and serving CookLang recipe files.

## What is CookLang?

CookLang is a markup language for writing recipes that makes them easy to read for humans and easy to parse for computers. It uses simple syntax to denote ingredients, cookware, timers, and cooking steps.

Example:
```
>> servings: 4

Add @water{2%cup} to a #pot and bring to boil ~{10%minutes}.
Add @pasta{350%g} and cook ~{8-10%minutes}.
```

Learn more at [cooklang.org](https://cooklang.org)

## Project Status

✅ **Phase 4 in Progress** - CI/CD pipeline and testing integration

**Phase 3 Complete**: Testing, deployment, and documentation
- Milestone 3.1: Docker Setup & Local Testing ✅
- Milestone 3.2: Comprehensive Integration Tests ✅
- Milestone 3.3: API Documentation & Testing Tools ✅

**Phase 4 in Progress**: CI/CD & Testing Integration
- Milestone 4.1: CI/CD Pipeline Setup ✅ (GitHub Actions with zero-cost hosted runners)
- Milestone 4.2: Integration Test Suite (planned)

**Tech Stack**: Rust + Axum + In-Memory Cache (DashMap) + Git storage + GitHub Actions CI/CD

## Goals

- Provide a RESTful API for recipe management
- Parse and validate CookLang recipe files (using official `cooklang-rs`)
- Store recipes in git repository for version control
- Support recipe search, filtering, and tagging
- Enable recipe scaling and shopping list generation
- Offer meal planning capabilities
- Multi-user support with JWT authentication

## Quick Start

### Prerequisites
- **VSCode + Dev Containers extension** (recommended for consistent dev environment)
- OR Rust 1.83+ ([install from rustup.rs](https://rustup.rs))
- Docker (for containerized deployment and testing without Rust)

### Development Setup (Recommended: DevContainer)

The easiest way to get started is using VSCode DevContainers:

1. Install [VSCode](https://code.visualstudio.com/) and the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. Clone and open the repository:
   ```bash
   git clone <your-repo-url>
   cd cooklang-backend
   code .
   ```
3. VSCode will prompt to "Reopen in Container" - click it
4. Once the container builds, run:
   ```bash
   cargo run
   ```

Server will start on `http://localhost:3000`

### Development Setup (Local Rust)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <your-repo-url>
cd cooklang-backend
cargo run

# Run tests
cargo test
```

### Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

### Docker Testing (Without Rust)

Test the Docker image without needing Rust installed:

```bash
# Run integration tests (builds image, tests API endpoints, cleans up)
scripts/docker-test.sh
```

This script:
- Creates an isolated test recipe repository in `/tmp`
- Builds the Docker image
- Starts a container with the test recipes
- Tests core API endpoints with curl
- Automatically cleans up resources

### Configuration

Create a `.env` file:
```
RECIPES_PATH=data/recipes
JWT_SECRET=your-secret-key-here
RUST_LOG=info
```

## API

The server provides a RESTful API for recipe management on `/api/v1`. 

**Quick Endpoints**:
- `GET /health` - Health check
- `GET /api/v1/status` - Server status and stats
- `POST /api/v1/recipes` - Create recipe
- `GET /api/v1/recipes` - List recipes (paginated)
- `GET /api/v1/recipes/:recipe_id` - Get recipe
- `PUT /api/v1/recipes/:recipe_id` - Update recipe
- `DELETE /api/v1/recipes/:recipe_id` - Delete recipe
- `GET /api/v1/recipes/search` - Search recipes by name
- `GET /api/v1/categories` - List categories
- `GET /api/v1/categories/:name` - Get recipes in category

See [docs/API.md](docs/API.md) for complete API documentation.

## Documentation

- [docs/API.md](docs/API.md) - Full REST API documentation and examples
- [docs/openapi.yaml](docs/openapi.yaml) - OpenAPI 3.0 specification (import into Swagger UI or other tools)
- [docs/postman-collection.json](docs/postman-collection.json) - Postman collection for API testing
- [docs/SAMPLE-RECIPES.md](docs/SAMPLE-RECIPES.md) - Sample recipes for testing API endpoints
- [docs/TESTING.md](docs/TESTING.md) - Testing guide (72 integration tests, Docker tests, CI/CD)
- [docs/DOCKER-TESTING.md](docs/DOCKER-TESTING.md) - Docker testing guide (requires only Docker, no Rust)
- [docs/CI-CD.md](docs/CI-CD.md) - GitHub Actions CI/CD pipeline documentation
- [PROJECT_PLAN.md](PROJECT_PLAN.md) - Architecture decisions and development roadmap
- [AGENTS.md](AGENTS.md) - Coding conventions and guidelines for AI agents

## Contributing

*Coming soon*

## License

*To be determined*
