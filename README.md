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

✅ **Phase 1 Complete** - Foundation established, ready for Phase 2 implementation.

**Tech Stack**: Rust + Axum + SQLite + Git storage

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
- OR Rust 1.75+ ([install from rustup.rs](https://rustup.rs))
- Docker (for containerized deployment)

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

### Configuration

Create a `.env` file:
```
DATABASE_URL=sqlite://data/db/cooklang.db
RECIPES_PATH=data/recipes
JWT_SECRET=your-secret-key-here
RUST_LOG=info
```

## Documentation

- [PROJECT_PLAN.md](PROJECT_PLAN.md) - Architecture decisions and development roadmap
- [AGENTS.md](AGENTS.md) - Coding conventions and guidelines for AI agents

## Contributing

*Coming soon*

## License

*To be determined*
