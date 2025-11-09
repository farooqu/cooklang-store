# Cooklang Store - Project Plan

**Status**: Phase 4 Complete, Phase 5 in Progress (Nov 9, 2025)

## Completed: Phase 1, 2, 3 & 4 ✅

**Phase 1**: Core architecture and recipe parsing  
**Phase 2**: Git-backed storage and REST API with full CRUD operations  
**Phase 3**: Testing infrastructure, Docker deployment, and comprehensive API documentation  
**Phase 4**: CI/CD & Testing Integration

Complete implementation:
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

- [ ] Maintain >80% test coverage
- [ ] Regular dependency updates
- [ ] Security vulnerability scanning
- [ ] API versioning strategy
- [ ] Backward compatibility considerations
- [ ] Performance optimization for large recipe collections (>1000 recipes)
