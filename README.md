# Cooklang Recipe Store

A self-hosted REST API for managing Cooklang recipe files. It provides a pluggable storage backend (disk or git) and a fast in-memory cache for browsing and searching your recipe collection.

## What is Cooklang?

Cooklang is a markup language for writing recipes that makes them easy to read for humans and easy to parse for computers. It uses simple syntax to denote ingredients, cookware, timers, and cooking steps.

Example:
```
>> servings: 4

Add @water{2%cup} to a #pot and bring to boil ~{10%minutes}.
Add @pasta{350%g} and cook ~{8-10%minutes}.
```

Learn more at [cooklang.org](https://cooklang.org)

## Project Status

🚀 **Early Alpha** - Core features implemented and tested, ready for self-hosted deployment

**Completed**:
- Full CRUD operations for Cooklang recipes
- Hierarchical categories with path support
- Git repository backend with version control
- 218 passing tests (unit + integration + doc tests)
- Docker deployment with docker-compose
- REST API with OpenAPI documentation
- JWT authentication framework
- CI/CD pipeline with GitHub Actions

**Tech Stack**: Rust 1.83+ + Axum + In-Memory Cache (DashMap) + Git storage + Docker + GitHub Actions

## Core Features

- **REST API**: Full CRUD operations for recipe management
- **Cooklang Parser**: Parse and validate Cooklang recipe files using the official `cooklang-rs` crate
- **Git Storage**: Store recipes in a git repository for version control and history
- **Hierarchical Categories**: Organize recipes into nested categories with full path support
- **In-Memory Cache**: Fast search and browsing with automatic cache invalidation
- **Docker Deployment**: Self-hosted containerized deployment with `docker-compose`
- **API Documentation**: Complete OpenAPI specification and Postman collection included

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
   cd cooklang-store
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
cd cooklang-store
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

Create a `.env` file (see `.env.example` for all options):
```
DATA_DIR=data/recipes
STORAGE_TYPE=disk  # 'disk' or 'git'
JWT_SECRET=your-secret-key-here
RUST_LOG=info
```

Or pass configuration via command-line arguments:
```bash
cooklang-store --data-dir /path/to/recipes --storage disk
```

**Storage Options:**
- `disk` (default): Direct filesystem storage - simple, no version history
- `git`: Git repository backend - provides version history, branching, and collaboration

## API

The server provides a RESTful API for recipe management on `/api/v1`. 

**Quick Endpoints**:
- `GET /health` - Health check
- `GET /api/v1/status` - Server status and stats
- `POST /api/v1/recipes` - Create recipe (content + path required)
- `GET /api/v1/recipes` - List recipes (paginated, returns RecipeSummary)
- `GET /api/v1/recipes/:recipe_id` - Get recipe (returns full RecipeResponse with content)
- `PUT /api/v1/recipes/:recipe_id` - Update recipe (content and/or path)
- `DELETE /api/v1/recipes/:recipe_id` - Delete recipe
- `GET /api/v1/recipes/search?q=...` - Search recipes by name
- `GET /api/v1/recipes/find-by-name?q=...` - Find recipes by name (fallback lookup)
- `GET /api/v1/recipes/find-by-path?path=...` - Find recipe by path (fallback lookup)
- `GET /api/v1/categories` - List categories
- `GET /api/v1/categories/:name` - Get recipes in category

See [docs/API.md](docs/API.md) for complete API documentation.

## Documentation

- [docs/API.md](docs/API.md) - Full REST API documentation and examples
- [docs/openapi.yaml](docs/openapi.yaml) - OpenAPI 3.0 specification (import into Swagger UI or other tools)
- [docs/postman-collection.json](docs/postman-collection.json) - Postman collection for API testing
- [tests/fixtures/](tests/fixtures/) - Test fixtures (Cooklang recipes for testing)
- [docs/TESTING.md](docs/TESTING.md) - Testing guide (72 integration tests, Docker tests, CI/CD)
- [docs/DOCKER-TESTING.md](docs/DOCKER-TESTING.md) - Docker testing guide (requires only Docker, no Rust)
- [docs/CI-CD.md](docs/CI-CD.md) - GitHub Actions CI/CD pipeline documentation
- [PROJECT_PLAN.md](PROJECT_PLAN.md) - Architecture decisions and development roadmap
- [AGENTS.md](AGENTS.md) - Coding conventions and guidelines for AI agents

## Roadmap

**Future Features** (not in v1):
- Full-text search and filtering
- Advanced recipe search (by ingredients, cooking time, etc.)
- Recipe images and step-level attachments
- Multi-user support with role-based access control
- Advanced authentication and permission management

See [PROJECT_PLAN.md](PROJECT_PLAN.md) for architecture decisions and the complete roadmap.

## Contributing

Contributions are welcome. Please open an issue or pull request.

## License

MIT
