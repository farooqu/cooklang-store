# CookLang Backend - Project Plan

**Status**: Phase 3 Complete, Phase 4 Next (Nov 9, 2025)

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

## Phase 4: Optimization & DevOps

### Milestones

#### Milestone 4.1: Performance & Monitoring
- Performance benchmarking
- Caching layer optimization (beyond basic in-memory)
- Rate limiting implementation
- Request logging and monitoring
- Response time optimization

#### Milestone 4.2: Deployment & Operations
- CI/CD pipeline setup
- Environment configuration management
- Git repository backup strategy
- Logging and error tracking
- Health check endpoints (may already exist)

#### Milestone 4.3: Testing Coverage
- Complete integration test suite
- Load testing
- Security audit
- Maintain >80% code coverage

## Phase 5: Search & Filtering

### Milestones

#### Milestone 5.1: Advanced Search
- Full-text search on recipe names and steps
- Filter by ingredients
- Filter by categories (already have listing, add filtering)
- Filter by cooking time
- Sort by date, name, etc.

#### Milestone 5.2: Shopping List Generation
- Shopping list generation from multiple recipes
- Ingredient unit conversion
- Aggregation and normalization

## Phase 6: Future Enhancements

### Deferred Features
- **Multi-user support & Authentication**: Can be added if self-hosting multiple users becomes a requirement
- **Import/Export**: Useful for recipe migration, but not core CRUD functionality
- **Meal Planning**: Belongs in frontend, not backend
- **Recipe Ratings/Reviews**: Social features deferred
- **OCR, AI Recommendations**: Advanced features for future consideration

## Technical Debt & Ongoing Tasks

- [ ] **DESIGN MISMATCH - Category Structure**: Current implementation treats categories as flat (single-level) - only the first subdirectory under `recipes/` is extracted as the category. A path like `recipes/meals/meat/traditional/chicken-biryani.cook` has category="meals", ignoring "meat/traditional". Need to decide:
  - Should categories support hierarchical nesting (e.g., "meals > meat > traditional")?
  - Or should the API structure restrict to `recipes/{category}/{slug}.cook` only?
  - Update implementation and documentation once decided
- [ ] Maintain >80% test coverage
- [ ] Regular dependency updates
- [ ] Security vulnerability scanning
- [ ] Performance monitoring and optimization
- [ ] API versioning strategy
- [ ] Backward compatibility considerations

## Future Considerations

- Mobile app integration
- Browser extension for recipe import
- OCR for scanning physical recipes
- AI-powered recipe recommendations
- Recipe variation tracking
- Ingredient substitution suggestions
- Integration with smart kitchen devices
