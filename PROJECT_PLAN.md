# CookLang Backend - Project Plan

## Phase 1: Foundation & Technology Selection

### Architectural Decisions Made

✅ **Storage Model**: Git repository as source of truth
- Recipe files stored as `.cook` files in a git repository
- All API changes (create/update/delete) persist to git with commits
- Enables version history, branching, and collaboration
- Users can edit recipes directly via git or use the API

✅ **Deployment**: Docker-based for self-hosting
- Primary audience: users self-hosting for themselves and family
- Simple Docker deployment with docker-compose
- Minimal configuration required

✅ **Language & Parser**: Rust with `cooklang-rs`
- Using official canonical CookLang parser from cooklang.org
- Rust offers performance, safety, and single binary deployment
- Parser libraries available: `cooklang-rs` (Rust), `cooklang-kotlin` (Kotlin)

✅ **Metadata Storage**: SQLite
- Lightweight, file-based database perfect for self-hosting
- Fast search indexing for recipes
- No separate database server needed

✅ **API Style**: REST
- Simple, well-understood, and widely supported
- Easy for self-hosted users to integrate

✅ **Web Framework**: Axum
- Modern, fast async framework built on tokio
- Excellent ergonomics and type safety
- Good ecosystem integration

✅ **Authentication**: JWT tokens
- Stateless authentication perfect for REST APIs
- Simple for self-hosted scenarios
- Easy to implement and understand

### Phase 1 Complete ✅
All architectural decisions made. Ready for implementation.

## Phase 2: Core Recipe Engine

### Milestones

#### Milestone 2.1: CookLang Parser Integration
- Integrate `cooklang` crate (v0.6) parser
- Create wrapper/adapter layer for recipe parsing
- Extract ingredients, cookware, timers, metadata, comments
- Handle parsing errors gracefully with user-friendly messages
- Unit tests for parser integration and edge cases

#### Milestone 2.2: Recipe Storage (Git + SQLite)
- Design SQLite schema for recipe metadata and search index
- Implement git repository initialization and management
- Store recipes as `.cook` files in git repository
- Store parsed recipe metadata in SQLite for fast querying
- Migration system for SQLite schema changes
- Sync strategy between git and database

#### Milestone 2.3: Git Integration Layer
- Implement git operations wrapper (commit, read, delete)
- Auto-commit recipe changes with meaningful commit messages
- Track file paths and git commit hashes in database
- Handle git merge conflicts gracefully
- Repository validation and error recovery

#### Milestone 2.4: Basic REST API
- Configure Axum routes and middleware
- Implement recipe CRUD endpoints:
  - `POST /api/v1/recipes` - Create recipe (writes .cook file + git commit)
  - `GET /api/v1/recipes` - List recipes with pagination
  - `GET /api/v1/recipes/:id` - Get single recipe
  - `PUT /api/v1/recipes/:id` - Update recipe (git commit)
  - `DELETE /api/v1/recipes/:id` - Delete recipe (git commit)
- Input validation and error handling
- Health check and status endpoints
- Basic API documentation structure

## Phase 3: Enhanced Features

#### Milestone 3.1: Search & Filtering
- Full-text search on recipe names and steps
- Filter by ingredients
- Filter by tags/categories
- Filter by cooking time
- Sort by date, name, rating, etc.

#### Milestone 3.2: Recipe Utilities
- Recipe scaling (adjust servings)
- Shopping list generation from multiple recipes
- Ingredient unit conversion
- Nutrition calculation (optional - may require external API)

#### Milestone 3.3: Tags & Organization
- Tag management (create, edit, delete)
- Assign multiple tags to recipes
- Tag-based browsing
- Collections/cookbooks (group recipes)

## Phase 4: User Management

#### Milestone 4.1: Authentication & Authorization
- User registration and login
- Password hashing and security
- JWT or session-based auth
- Role-based access control (admin, user)
- API key support for integrations

#### Milestone 4.2: Multi-User Support
- User-specific recipe libraries
- Recipe sharing (public/private/shared)
- User preferences and settings
- Recipe favorites/bookmarks

## Phase 5: Advanced Features

#### Milestone 5.1: Meal Planning
- Weekly meal planner
- Drag-and-drop meal scheduling
- Automatic shopping list from meal plan
- Meal plan templates

#### Milestone 5.2: Import/Export
- Import from other formats (JSON, plain text)
- Export recipes (PDF, JSON, CookLang)
- Bulk import/export
- Recipe backup and restore

#### Milestone 5.3: Social Features (Optional)
- Recipe ratings and reviews
- User comments
- Recipe sharing links
- Public recipe discovery

## Phase 6: Production Readiness

### Milestones

#### Milestone 6.1: Performance & Optimization
- Database indexing strategy
- Query optimization
- Caching layer (Redis or in-memory)
- Rate limiting
- Response time monitoring

#### Milestone 6.2: Deployment & DevOps
- Docker containerization
- CI/CD pipeline setup
- Environment configuration management
- Database backup strategy
- Logging and monitoring
- Health check endpoints

#### Milestone 6.3: Documentation & Testing
- Complete API documentation
- Integration test suite
- Load testing
- Security audit
- User documentation
- Deployment guide

## Technical Debt & Ongoing Tasks

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
