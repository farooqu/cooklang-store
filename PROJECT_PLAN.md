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

### Phase 1 Complete ✅ (Nov 8, 2025)
All architectural decisions made. Ready for implementation.

## Phase 2: Core Recipe Engine ✅ (Nov 8, 2025)

### Milestones

#### Milestone 2.1: CookLang Parser Integration ✅ (Completed Nov 8, 2025)
- ✅ Integrate `cooklang` crate (v0.6) parser
- ✅ Use cooklang types directly (ScalableRecipe, Ingredient, etc.)
- ✅ Simple parse_recipe() helper function with error handling
- ✅ Unit tests verifying parser integration works correctly

#### Milestone 2.2: Recipe Storage (Git + In-Memory Cache) ✅ (Completed Nov 8, 2025)

**Design Rationale**:
- Git is the persistent source of truth (provides durability, versioning, collaboration)
- In-memory cache (DashMap) for fast search and browsing
- Cache is rebuilt on startup by scanning git
- No external dependencies or migration management needed
- Perfect for self-hosted family scenario with <1000 recipes
- Simple to understand and deploy

**In-Memory Cache Structure**:
- `RecipeIndex`: Concurrent hashmap (DashMap) storing `git_path -> Recipe`
- Recipe metadata: name, description, category, ingredients list, cookware list, parsed content
- Secondary indexes:
  - By name (for search)
  - By category (for browsing)
  - By ingredients (for filtering)
- Cache is completely rebuilt from git on startup

**Git Repository Management**:
- Initialize git repo on first startup if missing
- Store recipes as `.cook` files with human-readable paths: `recipes/{category}/{subcategory}/{recipe-slug}.cook`
  - Categories and subcategories from recipe metadata or user input
  - Recipe slug derived from recipe name (lowercase, hyphens, no special chars)
  - Supports arbitrary nesting depth for organization
  - Example: `recipes/desserts/chocolate/triple-chocolate-cake.cook`
- Track full git path (relative to repo root) as canonical identifier in memory
- On startup: discover all `.cook` files recursively and populate cache
- Handle duplicate recipe names in same category by appending numeric suffix (e.g., `chocolate-cake-2.cook`)

**Cache Synchronization Strategy**:
- **Git is source of truth**: All recipe content lives in `.cook` files
- **In-memory cache is volatile**: Fast but lost on restart
- **Write-through pattern**: Update git first, then update in-memory cache
- **Startup**: Scan git and rebuild entire cache
- **Conflict resolution**: If cache is ever out of sync, rescan git and rebuild
- **Atomicity**: All write operations commit to git first; if git fails, operation fails; only then update cache

**Implementation Details**:
- `RecipeIndex` struct wrapping DashMap<String, Recipe>
- `RecipeRepository` struct managing all CRUD operations
- Methods: `create()`, `read()`, `update()`, `delete()`, `list()`, `search()`, `rebuild_from_git()`
- Each write operation: commit to git first, then update cache (git failure = operation failure)
- Cache provides: fast lookup, search by name/category/ingredients, filtering
- No database migrations or schema management needed

#### Milestone 2.3: Git Integration Layer ✅ (Completed Nov 8, 2025)
- ✅ Implement git operations wrapper (commit, read, delete)
- ✅ Auto-commit recipe changes with meaningful commit messages
- ✅ Track author/editor/contributor information in commit messages
- ✅ Support for optional comments in commit messages
- ✅ Support for author and comment tracking in all CRUD operations
- ⏳ Handle git merge conflicts gracefully (deferred to Phase 3)
- ⏳ Repository validation and error recovery (deferred to Phase 3)

**Implementation Details**:
- Git operations: `commit_file()`, `delete_file()`, `read_file()` with author variants
- Author-aware variants: `commit_file_with_author()`, `delete_file_with_author()`
- Repository methods support author and comment:
  - `create_with_author_and_comment()`, `update_with_author_and_comment()`, `delete_with_author_and_comment()`
  - All methods delegate through backward-compatible non-comment variants
- Commit message format: `"Action: details (by {author}) - {comment}"`
  - Author part optional: `(by {author})`
  - Comment part optional: `- {comment}`
  - Both can be provided, one, or neither

**Update scenarios with specialized commit messages**:
- Content only: `"Update recipe: Name (by Author)"`
- Rename only: `"Rename recipe: Old -> New (by Author)"`
- Move only (category): `"Move recipe: Name (old_cat -> new_cat) (by Author)"`
- Rename + move: `"Move recipe: Old -> New (to category) (by Author)"`
- Content + rename: `"Update recipe: New (renamed from Old) (by Author)"`
- Content + move: `"Update recipe: Name (moved to category) (by Author)"`
- All three: `"Update recipe: New (renamed from Old, moved to category) (by Author)"`

**Example commit messages**:
- `"Add recipe: Chocolate Cake"`
- `"Update recipe: Chocolate Cake (by Alice)"`
- `"Rename recipe: Old Name -> New Name (by Bob)"`
- `"Move recipe: Cake (desserts -> baking) (by Charlie)"`
- `"Move recipe: Old -> New (to baking) (by Dave)"`
- `"Delete recipe: Chocolate Cake (by Eve) - Duplicate entry"`

- Full test coverage: 37 tests including all update scenarios with author and comment tracking
- Backward-compatible: existing methods work without author/comment information

#### Milestone 2.4: Basic REST API ✅ (Completed Nov 8, 2025)
- ✅ Configure Axum routes and middleware
- ✅ Implement recipe CRUD endpoints:
  - `POST /api/v1/recipes` - Create recipe (writes .cook file + git commit)
  - `GET /api/v1/recipes` - List all recipes (with pagination)
  - `GET /api/v1/recipes/search` - Search recipes by name
  - `GET /api/v1/recipes/:recipe_id` - Get single recipe
  - `PUT /api/v1/recipes/:recipe_id` - Update recipe (git commit)
  - `DELETE /api/v1/recipes/:recipe_id` - Delete recipe (git commit)
- ✅ Implement category endpoints:
  - `GET /api/v1/categories` - List all categories
  - `GET /api/v1/categories/:name` - List recipes in category
- ✅ Recipe ID generation: SHA256 hash of git_path (first 12 hex chars, URL-friendly)
- ✅ Reverse lookup in cache: recipe_id -> git_path
- ✅ Input validation and error handling
- ✅ Health check and status endpoints
- ✅ Request/response models with proper serialization
- ✅ Error responses with context
- ✅ Pagination support with limit/offset
- ✅ CORS and body size limits configured
- ✅ RecipeRepository made thread-safe (Mutex wrapping git2::Repository)

**Implementation Details**:
- All endpoints fully functional and tested
- Recipe IDs are deterministic based on git_path (allows client-side ID generation if needed)
- Search and filtering use in-memory cache for fast performance
- All write operations include author and comment tracking from API
- API properly handles not-found errors and validation errors

## Phase 3: Testing & Deployment

### Milestones

#### Milestone 3.1: Docker Setup & Local Testing
- ✅ Dockerfile configured (already exists)
- [ ] Build and test Docker image locally
- [ ] Create test recipes in git repository for manual testing
- [ ] Verify docker-compose setup works end-to-end
- [ ] Test all CRUD endpoints with curl/Postman
- [ ] Verify git commits are recorded correctly
- [ ] Test error handling and edge cases

#### Milestone 3.2: Bug Fixes & Stability
- [ ] Fix any bugs discovered during testing
- [ ] Verify cache rebuilds correctly on startup
- [ ] Test with larger recipe collections
- [ ] Verify git repository initialization on first run
- [ ] Add logging for debugging

#### Milestone 3.3: API Documentation & Testing Tools
- [ ] Generate OpenAPI/Swagger specification
- [ ] Create Postman collection for manual testing
- [ ] Document all endpoints with examples
- [ ] Create sample recipes for testing
- [ ] Update README with quick start guide

### Phase 3 Complete Criteria
- Docker container runs without errors
- All CRUD operations verified working
- API is documented and testable
- Project can be deployed and used immediately

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
