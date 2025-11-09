# Cooklang Store - Project Plan

**Status**: Category Path Refactor Complete (Nov 9, 2025)

## Completed Implementation ✅
- Git repository as source of truth for recipe storage
- In-memory cache (DashMap) for fast search and browsing
- Rust with official `cooklang-rs` parser
- REST API with Axum on Docker
- JWT authentication framework
- Full CRUD operations with git commit tracking
- 218 tests (134 unit + 80 integration + 4 doc tests) - all passing
- Docker setup with docker-compose
- 12 REST API endpoints with OpenAPI specification (includes find-by-name, find-by-path)
- Postman collection with all new endpoints and working examples
- GitHub Actions CI/CD with test coverage reporting
- Hierarchical category support with full path preservation
- Category field refactored: replaced with explicit "path" and "fileName" fields (camelCase)
- Recipe titles derived from Cooklang YAML front matter metadata
- Automatic file renaming logic to keep disk names aligned with recipe titles
- Fallback lookup endpoints for ID migration when recipes are renamed

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

- [ ] **Description Field**: Response includes "description" field that's always null. Either remove it or extract actual description from .cook file metadata/content.

### Completed

- [x] **Category Field Semantics Refactor** (Completed Nov 9, 2025): Replaced "category" field with explicit "path" and "fileName" fields. Recipe titles now derived from Cooklang YAML front matter. File renaming logic implemented and tested. Added fallback lookup endpoints (find-by-name, find-by-path) for ID migration.

### Ongoing Tasks

- [ ] Maintain >80% test coverage
- [ ] Regular dependency updates
- [ ] Security vulnerability scanning
- [ ] API versioning strategy
- [ ] Backward compatibility considerations
- [ ] Performance optimization for large recipe collections (>1000 recipes)
