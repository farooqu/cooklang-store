# Agent Rules & Conventions

This document provides coding conventions, project structure, and common commands for AI coding agents working on this project.

## Project Overview

A self-hosted service for managing Cooklang recipe files. Cooklang is a markup language for recipes that makes them easy to read, write, and parse.

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
- Parser: **cooklang-rs** (canonical Cooklang parser from cooklang.org)
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
- ✅ **Caching**: In-memory DashMap (not SQLite) - simpler deployment, no migrations, cache rebuilds from storage on startup
- ✅ **Storage**: Pluggable backend architecture with DiskStorage (default) and GitStorage options - source of truth on filesystem + in-memory cache for queries
- ✅ **API**: REST (simple, self-hosted friendly)
- ✅ **Parser**: Official `cooklang-rs` crate (v0.6)
- ✅ **Recipe ID**: SHA256 hash of git_path (first 12 hex chars) - URL-friendly, deterministic, allows looking up recipes by ID in API while maintaining git_path internally
- ✅ **Thread Safety**: Git2::Repository wrapped in Mutex for DiskStorage in git mode; atomic operations per storage backend
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

**CRITICAL: Use Checklists to Track Milestone Implementation**:
- For each milestone being implemented, create a detailed checklist BEFORE starting work
- Use the checklist to organize the work into clear, verifiable tasks
- Structure it like: Core Implementation → Infrastructure → Documentation → Quality Standards → Verification
- Check off items as you complete them - this keeps progress visible and prevents scope creep
- Update relevant docs (PROJECT_PLAN.md, README.md, etc.) as you complete sections
- Once the milestone is complete, delete the checklist file (it served its purpose)
- This approach prevents context fragmentation and ensures nothing is forgotten

**CRITICAL: API Documentation Synchronization**:
When you modify ANY API endpoint, you MUST update these files in the same commit:
- [ ] `src/api/handlers.rs` - Endpoint implementation
- [ ] `docs/API.md` - Human-readable documentation with examples
- [ ] `docs/openapi.yaml` - Machine-readable OpenAPI spec
- [ ] `docs/postman-collection.json` - Test requests and examples
- [ ] `docs/SAMPLE-RECIPES.md` - If adding new testing scenarios

Failure to update all three files will cause documentation drift and confusion for future users.
**Do not consider the task complete until ALL documentation files are updated.**

## Documentation Files Reference

**Important**: When working on testing, deployment, or architectural tasks, consult the relevant documentation files:

| File | When to Consult | Contains |
|------|-----------------|----------|
| **docs/TESTING.md** | When adding tests, debugging tests, or setting up CI/CD | 24 integration tests with git verification, Docker tests, test helpers, CI/CD examples, coverage targets |
| **docs/DOCKER-TESTING.md** | When testing Docker image, preparing for deployment, or creating CI/CD pipeline | Docker test script guide, test coverage, debugging Docker tests, CI/CD integration |
| **docs/CI-CD.md** | When setting up GitHub Actions, monitoring pipelines, or extending CI/CD | GitHub Actions workflows, caching strategy, troubleshooting, cost analysis, extending workflows |
| **docs/API.md** | When adding API endpoints or documenting API changes | REST API endpoint documentation with examples |
| **docs/openapi.yaml** | When viewing/updating machine-readable API spec or importing into Swagger UI | Complete OpenAPI 3.0 specification of all endpoints, schemas, and responses |
| **docs/postman-collection.json** | When manually testing API endpoints or sharing testing tools | Postman collection with all endpoints, test requests, and environment variables |
| **docs/SAMPLE-RECIPES.md** | When testing API with realistic data or documenting Cooklang examples | 5+ sample recipes in Cooklang format with curl/Postman testing instructions |
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
  - Complete a phase (clean up implementation details, keep only high-level summary)
  - Make architectural decisions
  - Add or remove planned features
  - Identify new technical debt
  - Change project priorities or timeline
  
  **IMPORTANT - Keep Context Lean**: When marking a phase as complete, remove detailed implementation information (design rationale, detailed structures, code examples, etc.). Replace with a brief bullet-point summary of what was accomplished. This prevents context bloat for future AI agents. See the "Completed: Phase 1 & 2" section as a reference example.

- **docs/TESTING.md**: Update whenever you:
  - Add new test cases or test patterns
  - Change test coverage
  - Add testing utilities or helpers
  - Update CI/CD testing approach

- **docs/DOCKER-TESTING.md**: Update whenever you:
- Change Docker test script behavior
- Add new Docker test scenarios
- Update deployment testing approach

- **docs/API.md**: Update **MANDATORY** whenever you:
   - Add, modify, or remove API endpoints
   - Change request/response schemas
   - Update error codes or status codes
   - Add new query parameters or path parameters
   - Change authentication or authorization behavior
   - Add new examples or use cases
   - Include curl and code examples demonstrating the change
   - **Validation**: Ensure examples are accurate and executable

- **docs/openapi.yaml**: Update **MANDATORY** whenever you:
   - Add, modify, or remove API endpoints
   - Change request/response schemas
   - Update parameter definitions
   - Change HTTP status codes or error responses
   - This file is the machine-readable specification; keep it in sync with actual API
   - **Validation**: Run `python3 -c "import yaml; yaml.safe_load(open('docs/openapi.yaml'))"` to verify YAML syntax
   - Consider validating with Swagger UI or Insomnia

- **docs/postman-collection.json**: Update **MANDATORY** whenever you:
   - Add new API endpoints
   - Add new request/response examples
   - Change request body structure
   - Change query parameters
   - Update environment variable names
   - This enables manual testing; ensure examples are functional and current
   - **Validation**: Run `python3 -m json.tool docs/postman-collection.json > /dev/null` to verify JSON syntax

- **docs/SAMPLE-RECIPES.md**: Update whenever you:
   - Add new API testing scenarios that aren't covered by existing samples
   - Update example recipes or curl commands if behavior changes
   - Change testing procedures
   - Add new Cooklang syntax examples

### Documentation Update Rule

After implementing ANY feature or making significant changes:
1. Review which documentation files are affected
2. Update those files BEFORE marking the task complete
3. Ensure examples and instructions remain accurate
4. Keep the docs concise and current - remove outdated information

### API Documentation Synchronization Checklist

**For every API endpoint change**, complete this checklist before submitting:

- [ ] **Code** (`src/api/handlers.rs` or `src/api/mod.rs`): Endpoint implemented and tested
- [ ] **API.md**: Updated with endpoint description, parameters, request/response examples, and error codes. Include curl examples.
- [ ] **openapi.yaml**: Updated with endpoint schema, validated YAML syntax with Python
- [ ] **postman-collection.json**: Added/updated request examples, validated JSON syntax with Python
- [ ] **SAMPLE-RECIPES.md**: Added example if it demonstrates new functionality (optional)
- [ ] All files reviewed and examples tested to be accurate
- [ ] Files formatted and validated before commit

**Why this matters**: Users rely on Postman collection and API.md for integration. OpenAPI is machine-readable. Outdated docs cause wasted debugging time and frustration.

## Cooklang Specification

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
