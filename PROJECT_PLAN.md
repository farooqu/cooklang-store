# Cooklang Store - Project Plan

**Status**: Phase 3 Complete, Phase 4 in Progress (Nov 9, 2025)

## Completed: Phase 1, 2 & 3 ✅

**Phase 1**: Core architecture and recipe parsing  
**Phase 2**: Git-backed storage and REST API with full CRUD operations  
**Phase 3**: Testing infrastructure, Docker deployment, and comprehensive API documentation

Complete implementation:
- Git repository as source of truth for recipe storage
- In-memory cache (DashMap) for fast search and browsing
- Rust with official `cooklang-rs` parser
- REST API with Axum on Docker
- JWT authentication framework
- Full CRUD operations with git commit tracking
- 37+ unit tests and 24+ integration tests
- Docker setup with docker-compose
- 10 REST API endpoints with OpenAPI specification
- Postman collection and sample recipes for testing

## Phase 4: CI/CD & Testing Integration

### Milestones

#### Milestone 4.1: CI/CD Pipeline Setup ✅ (Nov 9, 2025)
- GitHub Actions workflows for Rust tests, linting, and security audits
- Docker image building, linting, and integration testing
- Code coverage reporting with Codecov integration
- Automated caching for faster builds (cargo, Docker layers)
- Zero-cost CI/CD using GitHub-hosted runners (ubuntu-latest)
- Comprehensive documentation in docs/CI-CD.md

#### Milestone 4.2: Integration Test Suite
- Complete integration test suite (24+ tests)
- Automated test execution in CI/CD
- Docker-based test environment
- Test coverage verification (maintain >80%)

## Phase 5: Search & Filtering

### Milestones

#### Milestone 5.1: Search & Filter by Name
- Full-text search on recipe names
- Fuzzy matching for typo tolerance

#### Milestone 5.2: Filter by Ingredients
- Filter recipes by ingredient presence
- Support ingredient-based queries

#### Milestone 5.3: Filter by Categories
- Filter recipes by category
- Category listing with filtering

## Completed Deferred Features ✅

### Storage Flexibility (Completed)
- Support persisting recipes directly to disk without git
- Git storage backend with automatic commits
- Pluggable RecipeStorage trait allows easy addition of new backends
- Default to disk (simpler), git available when needed via COOKLANG_STORAGE_TYPE env var

## Deferred Features

Features not critical to the core purpose of CRUD operations on `.cook` files:

### Advanced Searching & Filtering
- Full-text search on recipe steps
- Filter by cooking time
- Sort by date, name, and other fields
- Advanced filtering combinations

### Operational Enhancements
- Environment configuration management
- Health check endpoints
- Request logging and monitoring

### Future Ideas
- Recipe images (store alongside `.cook` files)
- Step-level images (attach images to specific recipe steps)

### Out of Scope
The following are intentionally excluded as they go beyond the core purpose of persisting raw `.cook` files:
- Rate limiting, performance benchmarking, load testing
- Shopping list generation, ingredient conversion, import/export
- Multi-user support, advanced authentication
- Recipe ratings/reviews, meal planning
- Mobile apps, OCR, AI recommendations, smart kitchen integrations

## Technical Debt & Ongoing Tasks

- [ ] **DESIGN MISMATCH - Category Structure**: Current implementation treats categories as flat (single-level) - only the first subdirectory under `recipes/` is extracted as the category. A path like `recipes/meals/meat/traditional/chicken-biryani.cook` has category="meals", ignoring "meat/traditional". Need to decide:
- Should categories support hierarchical nesting (e.g., "meals > meat > traditional")?
- Or should the API structure restrict to `recipes/{category}/{slug}.cook` only?
- Update implementation and documentation once decided
- [ ] Maintain >80% test coverage
- [ ] Regular dependency updates
- [ ] Security vulnerability scanning
- [ ] API versioning strategy
- [ ] Backward compatibility considerations
