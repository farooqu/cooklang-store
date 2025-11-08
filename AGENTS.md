# Agent Rules & Conventions

This document provides coding conventions, project structure, and common commands for AI coding agents working on this project.

## Project Overview

A self-hosted backend service for managing CookLang recipe files. CookLang is a markup language for recipes that makes them easy to read, write, and parse.

**Target Audience**: Users self-hosting for themselves and their family.

## Key Architectural Decisions

### Git as Source of Truth
- Recipe files (`.cook` format) are stored in a git repository
- All API operations (create/update/delete) commit changes to git
- Provides version history, branching, and collaboration capabilities
- Users can edit recipes via API or directly in git

### Deployment Model
- Docker-based deployment with docker-compose
- Designed for simple self-hosting scenarios
- Minimal configuration required

## Technology Stack

**Selected**:
- Language: **Rust** (use official `cooklang-rs` parser library)
- Parser: **cooklang-rs** (canonical CookLang parser from cooklang.org)
- Metadata DB: **SQLite** (lightweight, file-based, perfect for self-hosting)
- API: **REST** (simple, well-understood)
- Storage: **Git repository** for recipe files + SQLite for search index/metadata

## Code Conventions

### General Guidelines

- Write clean, readable code with meaningful variable names
- Follow the language-specific style guide once the tech stack is chosen
- Keep functions small and focused on a single responsibility
- Write unit tests for all business logic
- Include integration tests for API endpoints
- Use dependency injection for testability
- Handle errors explicitly - no silent failures

### API Design

- Use consistent naming conventions for endpoints
- Return appropriate HTTP status codes
- Include proper error messages with context
- Version the API (e.g., `/api/v1/recipes`)
- Document all endpoints with OpenAPI/Swagger

### Security

- Validate all inputs
- Sanitize file paths to prevent directory traversal
- Implement rate limiting
- Use authentication and authorization
- Never log sensitive data
- Use environment variables for secrets

### Testing

- Aim for >80% code coverage
- Write tests before fixing bugs (TDD for bug fixes)
- Use descriptive test names that explain the scenario
- Mock external dependencies

## Project Structure

```
/
├── src/           # Source code
├── tests/         # Test files
├── docs/          # Documentation
├── scripts/       # Build and deployment scripts
├── config/        # Configuration files
└── migrations/    # Database migrations (if applicable)
```

## Common Commands

### Development
- **Build**: `cargo build`
- **Run dev server**: `cargo run`
- **Test**: `cargo test`
- **Test with output**: `cargo test -- --nocapture`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`
- **Check without building**: `cargo check`

### Production
- **Build release**: `cargo build --release`
- **Run with Docker**: `docker-compose up -d`
- **View logs**: `docker-compose logs -f`
- **Stop**: `docker-compose down`

### Database
- **Run migrations**: `sqlx migrate run`
- **Create migration**: `sqlx migrate add <name>`

## Development Workflow

**BEFORE starting any work**: Consult [PROJECT_PLAN.md](PROJECT_PLAN.md) to understand:
- Current project phase and priorities
- Architectural decisions already made
- Which milestones are complete or in progress
- Technical debt and known issues

**When implementing features**:
1. Create feature branch from `main`
2. Write tests first (TDD approach)
3. Implement feature
4. Run linter and tests
5. Update documentation (see Documentation Maintenance below)
6. Create pull request

## Documentation Maintenance

**IMPORTANT**: AI agents MUST keep documentation in sync with code changes.

### When to Update Documentation

- **AGENTS.md**: Update whenever you:
  - Add new common commands (build, test, lint, etc.)
  - Establish new coding patterns or conventions
  - Make technology stack decisions
  - Add new project structure directories
  - Change development workflow

- **README.md**: Update whenever you:
  - Change project goals or features
  - Add quick start instructions
  - Update project status or milestones
  - Add new documentation files
  - Change installation or setup steps

- **PROJECT_PLAN.md**: Update whenever you:
  - Complete a milestone (mark as done with date)
  - Make architectural decisions
  - Add or remove planned features
  - Identify new technical debt
  - Change project priorities or timeline

### Documentation Update Rule

After implementing ANY feature or making significant changes:
1. Review which documentation files are affected
2. Update those files BEFORE marking the task complete
3. Ensure examples and instructions remain accurate
4. Keep the docs concise and current - remove outdated information

## CookLang Specification

Reference: https://cooklang.org/docs/spec/

Key concepts to support:
- **Ingredients**: `@ingredient{quantity%unit}`
- **Cookware**: `#cookware{}`
- **Timers**: `~timer{duration}`
- **Comments**: `--` or `[- -]`
- **Metadata**: Key-value pairs at the start of the file
- **Steps**: Text instructions with embedded components

## API Features to Support

- CRUD operations for recipes
- Search and filter recipes
- Tag management
- Import/export recipes
- Recipe scaling (adjust servings)
- Shopping list generation
- Meal planning
- User management and authentication

## Performance Considerations

- Cache parsed recipes
- Index recipes for fast search
- Optimize file I/O operations
- Consider lazy loading for large recipe collections
- Implement pagination for list endpoints
