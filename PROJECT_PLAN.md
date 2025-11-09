# Cooklang Store - Project Plan

**Status**: Phase 4 Complete, Phase 5 in Progress (Nov 9, 2025)

## Completed Implementation ✅
- Git repository as source of truth for recipe storage
- In-memory cache (DashMap) for fast search and browsing
- Rust with official `cooklang-rs` parser
- REST API with Axum on Docker
- JWT authentication framework
- Full CRUD operations with git commit tracking
- 161 tests (55 unit + 31 API + 25 disk storage + 25 git storage)
- Docker setup with docker-compose
- 10 REST API endpoints with OpenAPI specification
- Postman collection and sample recipes for testing
- GitHub Actions CI/CD with test coverage reporting
- Hierarchical category support with full path preservation

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

### Known Issues

- [ ] **Docker Test Failure**: Create Recipe test fails in GitHub Actions with error "Failed to write file: recipes/desserts/vanilla-cake.cook". Need to investigate file creation logic, permissions, or git configuration in Docker environment.

- [ ] **Description Field**: Response includes "description" field that's always null. Either remove it or extract actual description from .cook file metadata/content.

### In Progress

- [ ] **Category Field Semantics Refactor** (See MILESTONE_CATEGORY_PATH_REFACTOR.md): Replace "category" field with proper "path" and "file_name" fields. Derive recipe names from Cooklang metadata. Implement file renaming logic to keep disk names aligned with recipe titles.

### Ongoing Tasks

- [ ] Maintain >80% test coverage
- [ ] Regular dependency updates
- [ ] Security vulnerability scanning
- [ ] API versioning strategy
- [ ] Backward compatibility considerations
- [ ] Performance optimization for large recipe collections (>1000 recipes)
