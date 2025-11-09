# Agent Rules & Conventions

This document provides coding conventions, project structure, and common commands for AI coding agents working on this project.

## Project Overview

A self-hosted backend service for managing CookLang recipe files. CookLang is a markup language for recipes that makes them easy to read, write, and parse.

**Target Audience**: Users self-hosting for themselves and their family.

**Design Philosophy**: Keep it minimal. Use the official cooklang crate directly without unnecessary wrapper layers.

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
- Caching: **In-memory (DashMap)** (fast, volatile, rebuilt from git on startup)
- API: **REST** (simple, well-understood)
- Storage: **Git repository** for recipe files + in-memory cache for search/index

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
- **Test specific module**: `cargo test parser`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`
- **Check without building**: `cargo check`

### Production & Deployment
- **Build release**: `cargo build --release`
- **Run with Docker**: `docker-compose up -d`
- **View logs**: `docker-compose logs -f`
- **Stop**: `docker-compose down`
- **Test Docker image**: `scripts/docker-test.sh` (requires Docker, no Rust needed)

## Development Workflow

**BEFORE starting any work**: Consult [PROJECT_PLAN.md](PROJECT_PLAN.md) to understand:
- Current project phase and priorities
- Architectural decisions already made
- Which milestones are complete or in progress
- Technical debt and known issues

**CRITICAL: Review and Refine the Plan First**:
- Do NOT proceed to implementation until the plan is clear and detailed
- For each milestone being implemented, analyze:
  - Is it aligned with project goals and philosophy?
  - Are requirements specific enough (data structures, APIs, behaviors)?
  - Are edge cases and error handling covered?
  - Is the scope reasonable for one milestone?
- Propose revisions to PROJECT_PLAN.md with specific, actionable details
- Get explicit approval before writing code
- This prevents rework and keeps the codebase coherent

**CRITICAL: Question Technology Choices Anytime**:
- For any major technology decision (at any phase), you may explicitly ask:
  - **"Is this actually needed?"** - Challenge assumptions
  - **"What's the simplest solution?"** - Align with "minimal" philosophy
  - **"What are the trade-offs?"** - Deployment complexity, code complexity, operational burden
  - **"Are there simpler alternatives?"** - In-memory vs SQLite, async vs sync, etc.
- Self-hosted family scenario: prefer simple, zero-config solutions over feature-rich ones

**CRITICAL: Document Reasoning for Decisions**:
- When you question a technology choice, explicitly decide: keep it or change it?
- **If keeping it**: Update AGENTS.md "Documented Architectural Decisions" with the reasoning for why you rejected the alternative
- **If changing it**: Update PROJECT_PLAN.md and AGENTS.md with the new decision and rationale
- Include in the reasoning: trade-offs considered, why this choice serves the project goals better
- This prevents the same question from being asked repeatedly because the reasoning is captured
- Example: "✅ **Caching**: In-memory DashMap (not SQLite) - simpler deployment, no migrations needed, cache rebuilds from git on startup which is fast for typical family-scale collections"

**Documented Architectural Decisions**:
- ✅ **Caching**: In-memory DashMap (not SQLite) - simpler deployment, no migrations, cache rebuilds from git on startup
- ✅ **Storage**: Git repository as source of truth + in-memory cache for queries
- ✅ **API**: REST (simple, self-hosted friendly)
- ✅ **Parser**: Official `cooklang-rs` crate (v0.6)
- ✅ **Recipe ID**: SHA256 hash of git_path (first 12 hex chars) - URL-friendly, deterministic, allows looking up recipes by ID in API while maintaining git_path internally
- ✅ **Thread Safety**: git2::Repository wrapped in Mutex to allow Arc<RecipeRepository> in Axum state
- ✅ **Rust Version**: 1.83+ (required for Cargo.lock v4 format used in dependencies)

**API Module Structure** (`src/api/`):
- `mod.rs`: Router builder and route definitions
- `handlers.rs`: HTTP request handlers for all endpoints
- `models.rs`: Request/response models and query parameters
- `responses.rs`: Serializable response types and error responses

**When implementing features**:
1. Create feature branch from `main`
2. Write tests first (TDD approach)
3. Implement feature
4. Run linter and tests
5. Update documentation (see Documentation Maintenance below)
6. Create pull request

## Documentation Files Reference

**Important**: When working on testing, deployment, or architectural tasks, consult the relevant documentation files:

| File | When to Consult | Contains |
|------|-----------------|----------|
| **docs/TESTING.md** | When adding tests, debugging tests, or setting up CI/CD | 24 integration tests with git verification, Docker tests, test helpers, CI/CD examples, coverage targets |
| **docs/DOCKER-TESTING.md** | When testing Docker image, preparing for deployment, or creating CI/CD pipeline | Docker test script guide, test coverage, debugging Docker tests, CI/CD integration |
| **docs/API.md** | When adding API endpoints or documenting API changes | REST API endpoint documentation with examples |
| **PROJECT_PLAN.md** | Before starting ANY work | Current project phase, completed milestones, architectural decisions, technical debt |
| **README.md** | When updating installation, quick start, or project status | Quick start guide, project status, feature list, deployment instructions |

## Documentation Maintenance

**IMPORTANT**: AI agents MUST keep documentation in sync with code changes.

### When to Update Documentation

- **AGENTS.md**: Update whenever you:
  - Add new common commands (build, test, lint, etc.)
  - Establish new coding patterns or conventions
  - Make technology stack decisions
  - Add new project structure directories
  - Change development workflow
  - Create new .md documentation files (add to reference table above)

- **README.md**: Update whenever you:
  - Change project goals or features
  - Add quick start instructions
  - Update project status or milestones
  - Add new documentation files (update links)
  - Change installation or setup steps

- **PROJECT_PLAN.md**: Update whenever you:
  - Complete a milestone (mark as done with date)
  - Make architectural decisions
  - Add or remove planned features
  - Identify new technical debt
  - Change project priorities or timeline

- **docs/TESTING.md**: Update whenever you:
  - Add new test cases or test patterns
  - Change test coverage
  - Add testing utilities or helpers
  - Update CI/CD testing approach

- **docs/DOCKER-TESTING.md**: Update whenever you:
  - Change Docker test script behavior
  - Add new Docker test scenarios
  - Update deployment testing approach

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
