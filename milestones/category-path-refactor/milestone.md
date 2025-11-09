# Milestone: Category Field Semantics & Path Handling Refactor

**Status**: Phases 1-2 Complete | Nov 9, 2025

**Goal**: Clarify API to properly represent recipe locations and derive titles from content metadata. Replace ambiguous "category" field with explicit "path" and "file_name" fields. Move source of truth for recipe names to Cooklang metadata.

**Design Decisions**:
- Recipe names (titles) are derived from Cooklang YAML front matter (`title` field), not stored separately in API
- YAML front matter format: block delimited by `---` at file start (e.g., `---\ntitle: Recipe Name\n---`)
- File names on disk are generated from recipe titles and kept in sync
- "path" represents directory location (no `recipes/` prefix, relative to data-dir)
- Write operations only accept `path` and `content`; everything else is read-only or derived
- File renaming aligns disk name with title on every write operation
- Cooklang content MUST include YAML front matter with `title` field (validated on create/update)

**API Response Formats** (camelCase for modern REST API convention):

*Full Recipe Response* (RecipeResponse):
```json
{
  "recipeId": "a1b2c3d4e5f6",
  "recipeName": "Chocolate Cake",
  "path": "desserts",
  "fileName": "chocolate-cake.cook",
  "content": "# Chocolate Cake\n...",
  "description": null
}
```
Note: `description` omitted from JSON if null (via `#[serde(skip_serializing_if = "Option::is_none")]`)

*Recipe Summary* (RecipeSummary) - used in lists/search:
```json
{
  "recipeId": "a1b2c3d4e5f6",
  "recipeName": "Chocolate Cake",
  "path": "desserts"
}
```
Note: No `fileName` or `content` in summaries; `description` omitted if null

**Write Payloads**:
- **Create**: `{ "content": "...", "path": "desserts?" }`  
  - content required, must include YAML front matter with `title` field
  - path optional (defaults to root)
  - author/comment optional
  - Example content:
    ```
    ---
    title: Chocolate Cake
    ---
    
    # Instructions here...
    @flour{2%cups}
    ```
- **Update**: `{ "content": "...", "path": "desserts?" }`
  - At least one required
  - If content provided, must include YAML front matter with `title` field
  - author/comment optional

---

## Phase 1: API Specification ✅ COMPLETE

**Completed** (Nov 9, 2025):
- Updated OpenAPI spec with new RecipeResponse, RecipeSummary, CreateRecipeRequest, UpdateRecipeRequest schemas
- Added fallback lookup endpoints: `GET /api/v1/recipes/find-by-name` and `GET /api/v1/recipes/find-by-path`
- Updated API.md documentation with YAML front matter requirements, Recipe ID stability section, and file name generation rules
- Updated Postman collection with new request/response formats and fallback endpoint requests
- All changes use camelCase for JSON fields (recipeId, recipeName, fileName, etc.)

---

## Phase 2: Core Business Logic ✅ COMPLETE

**Completed** (Nov 9, 2025):
- Added `extract_recipe_title()` to parse YAML front matter and extract recipe titles
- Added `generate_filename()` and `normalize_path()` utilities for filename generation and path validation
- Added `should_rename_file()` to detect when files need renaming
- Updated repository layer (`create()`, `update()`, `rebuild_from_storage()`) to handle title extraction, filename generation, and file renaming
- Added comprehensive unit tests (134 total) covering title extraction, filename generation, rename detection, and repository operations
- Added integration tests (70 total) validating YAML front matter requirements and file renaming behavior

---

## Phase 3: API Response Layer

### Task 3.1: Update Response Structs (responses.rs)

**RecipeResponse** (full response with content):
- [ ] Rename `RecipeResponse.name` → `recipe_name` (with `#[serde(rename = "recipeName")]`)
- [ ] Add `RecipeResponse.path: Option<String>`
- [ ] Add `RecipeResponse.file_name: String` (with `#[serde(rename = "fileName")]`)
- [ ] Keep `RecipeResponse.description: Option<String>` (with `#[serde(skip_serializing_if = "Option::is_none")]`)
- [ ] Keep `RecipeResponse.content: String`
- [ ] Keep `RecipeResponse.recipe_id: String` (with `#[serde(rename = "recipeId")]`)

**RecipeSummary** (summaries for lists/search):
- [ ] Rename `RecipeSummary.name` → `recipe_name` (with `#[serde(rename = "recipeName")]`)
- [ ] Add `RecipeSummary.path: Option<String>`
- [ ] Remove `RecipeSummary.file_name` (not in summaries)
- [ ] Remove `RecipeSummary.description` (not in summaries, or add with skip_if_none if useful for filtering)
- [ ] Keep `RecipeSummary.recipe_id: String` (with `#[serde(rename = "recipeId")]`)

**Implementation notes**:
- [ ] Use `#[serde(rename = "...")]` to map snake_case Rust to camelCase JSON
- [ ] Use `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields to omit nulls

### Task 3.2: Update Request Structs (models.rs)
- [ ] Replace `CreateRecipeRequest` fields: keep only `content`, `path`, `author`, `comment`
- [ ] Replace `UpdateRecipeRequest` fields: keep only `content`, `path`, `author`, `comment` (all optional)
- [ ] Add validation in structs or handlers to ensure: create has content, update has at least one of content/path
- [ ] Remove or deprecate any fields no longer needed (`name`, `category` from old design)

---

## Phase 4: API Handlers

### Task 4.1: Update Create Handler
- [ ] Parse `CreateRecipeRequest`: extract `content`, `path`, `author`, `comment`
- [ ] Validate: `content` is required and non-empty
- [ ] Validate: `content` is valid Cooklang with YAML front matter containing `title` field
- [ ] Extract title from content YAML front matter (via `extract_recipe_title()`)
- [ ] Generate filename from title
- [ ] Default `path` to empty string (root) if not provided
- [ ] Call repository `create()` with: path, filename, content, author, comment
- [ ] Return full `RecipeResponse` with recipeName, path, fileName derived from created recipe
- [ ] Return 400 Bad Request if title missing from front matter
- [ ] Add/update tests

### Task 4.2: Update Update Handler
- [ ] Parse `UpdateRecipeRequest`: extract `content`, `path`, `author`, `comment`
- [ ] Validate: at least one of `content` or `path` provided
- [ ] If updating content: validate it's valid Cooklang with YAML front matter `title` field
- [ ] If updating content: extract new title from YAML front matter, generate new filename
- [ ] Call repository `update()` with path, content, author, comment
- [ ] Return full `RecipeResponse` with updated values
- [ ] Handle file renaming internally in repository (if title changed or name misaligned)
- [ ] Return 400 Bad Request if content provided but missing title in front matter
- [ ] Add/update tests

### Task 4.3: Update List Handlers
- [ ] Update `list_recipes()` to return `RecipeSummary` with new fields (recipeName, path, fileName, no description, no content)
- [ ] Update `search_recipes()` with same changes
- [ ] Update `get_category_recipes()` with same changes
- [ ] Build path from git_path when creating responses
- [ ] Extract fileName from git_path
- [ ] Extract recipeName from content metadata (may need to read from disk for each recipe, consider performance)
- [ ] Add/update tests

### Task 4.4: Update Get Single Recipe Handler
- [ ] Return full `RecipeResponse` with all fields (recipeName, path, fileName, description, content)
- [ ] Description field should remain null unless implemented in Phase 6+

### Task 4.5: Add Fallback Lookup Endpoints (for ID stability migration)
These endpoints help clients find recipes when IDs change due to renames.

**Add handler: find_recipe_by_name**
- [ ] Endpoint: `GET /api/v1/recipes/find-by-name?q=Chocolate%20Cake`
- [ ] Search across recipe names (recipeName from metadata)
- [ ] Return array of matching recipes as `RecipeSummary` (similar to search endpoint)
- [ ] Support pagination: `limit`, `offset` parameters
- [ ] Returns 200 with empty array if no matches (not 404)

**Add handler: find_recipe_by_path**
- [ ] Endpoint: `GET /api/v1/recipes/find-by-path?path=desserts`
- [ ] Find recipe at exact path (if exists)
- [ ] Return single `RecipeSummary` if found
- [ ] Return 404 if path not found
- [ ] Path is optional (root) and does not include `recipes/` prefix

**Add to router** (src/api/mod.rs):
- [ ] Wire endpoints into router
- [ ] Document that these are fallback/recovery endpoints for ID changes

---

## Phase 5: Documentation & Examples

### Task 5.1: Update Postman Collection
- [ ] Add/update Create Recipe request with new payload format (content + path + author + comment)
- [ ] Add/update Update Recipe request with new payload format (optional content and/or path)
- [ ] Add new requests for fallback endpoints:
  - [ ] `GET /recipes/find-by-name?q=Chocolate`
  - [ ] `GET /recipes/find-by-path?path=desserts`
- [ ] Update all response examples to show new RecipeResponse and RecipeSummary schemas
- [ ] Add examples for path parameter (empty for root, hierarchical paths like `desserts/chocolate`)
- [ ] Add example showing ID change scenario: create recipe → rename via title → lookup by name
- [ ] Validate JSON syntax: `python3 -m json.tool docs/postman-collection.json > /dev/null`

### Task 5.2: Update Sample Recipes
- [ ] Ensure all sample recipes have proper Cooklang metadata with titles
- [ ] Update curl examples to match new request format
- [ ] Add example showing file renaming behavior
- [ ] Test all examples work with actual API

### Task 5.3: Update README if Needed
- [ ] Verify quick start examples work with new API
- [ ] Update any architecture notes about recipe naming

---

## Phase 6: Testing & Verification

### Task 6.1: Unit Tests
- [ ] Test title extraction from Cooklang YAML front matter (various formats)
  - [ ] Standard format:
    ```
    ---
    title: Recipe Name
    ---
    ```
  - [ ] Case-insensitive key lookup (Title, TITLE, etc.)
  - [ ] Missing title returns error
  - [ ] Malformed YAML front matter handling (missing `---`, invalid YAML)
- [ ] Test filename generation (normal, special chars, unicode, edge cases)
- [ ] Test path normalization
- [ ] Test file renaming detection logic
- [ ] Verify misaligned files are renamed on update
- [ ] Validate Cooklang content validation (must have front matter title)

### Task 6.2: Integration Tests
- [ ] Create recipe with default path (root) - content must have YAML front matter `title`
- [ ] Create recipe with hierarchical path - YAML front matter required
- [ ] Create recipe with missing title in YAML front matter - returns 400 Bad Request
- [ ] Update recipe: only content (should rename file if title changed)
  - [ ] Old title: "Chocolate Cake", new title: "Dark Chocolate Cake" → file renamed
- [ ] Update recipe: only path (should move to new directory, keep title/filename)
- [ ] Update recipe: both path and content with new title
- [ ] Verify file alignment: created file name matches generated name from title
- [ ] Test Git history tracks renames correctly (git mv preserves history)
- [ ] List/search returns correct recipe_name, path, file_name for all recipes
- [ ] Test find-by-name endpoint: exact match and partial match
- [ ] Test find-by-name pagination: limit and offset
- [ ] Test find-by-path endpoint: finds recipe at exact path
- [ ] Test find-by-path with root path (empty string or omitted)
- [ ] Test find-by-path returns 404 for non-existent path
- [ ] **Scenario: ID change on rename**
  - [ ] Create recipe with title "Chocolate Cake" → capture ID
  - [ ] Update recipe content with new title "Dark Chocolate Cake" → file renamed
  - [ ] Verify old ID returns 404
  - [ ] Verify find-by-name returns recipe with new ID
  - [ ] Verify new ID can be used for subsequent operations

### Task 6.3: API Contract Tests
- [ ] Postman collection runs all tests successfully
- [ ] All response schemas match OpenAPI spec
- [ ] Status endpoint shows correct recipe count and categories
- [ ] Edge cases: empty path, deeply nested paths, special characters in titles

### Task 6.4: Coverage & Quality
- [ ] Achieve >80% code coverage on new/modified functions
- [ ] Run `cargo clippy` and `cargo fmt`
- [ ] Verify no regressions in existing tests
- [ ] Docker tests pass

---

## Definition of Done

- [ ] All API spec files updated (openapi.yaml, API.md, postman-collection.json, SAMPLE-RECIPES.md)
- [ ] All response/request structs refactored with camelCase + serde attributes
- [ ] All handlers updated (create, update, list, get) and tested
- [ ] Title extraction and filename generation working correctly
- [ ] File renaming logic handles misalignment and updates on every write
- [ ] Fallback lookup endpoints implemented (find-by-name, find-by-path)
- [ ] Recipe ID stability documented and tested (Option 4: accept with migration tools)
- [ ] >80% test coverage on new/modified functions
- [ ] All integration tests passing (including ID change scenario)
- [ ] Postman collection includes all new endpoints with working examples
- [ ] API.md documents Recipe ID behavior and fallback endpoints
- [ ] Documentation accurate and all examples tested
- [ ] PROJECT_PLAN.md updated with completion notes
- [ ] No regressions in other functionality
- [ ] `cargo clippy` and `cargo fmt` pass with no issues

---

## Implementation Notes

**Performance Consideration**: Extracting recipe names from content metadata during list operations may require reading files from disk. Consider:
- Caching extracted names in memory (DashMap) during cache rebuild
- Or accept the disk read cost for accuracy
- Or store in git metadata (git notes) for faster access

**Git Handling**: File renames in git should:
- Use git mv to preserve history
- Update cache appropriately
- Ensure recipe_id remains stable (based on git_path, which changes on rename - may be issue)

**Recipe ID Stability (DECISION: Option 4 - Accept with Migration Tools)**:

Current design: `recipe_id = SHA256(git_path)[..12]` (see `src/cache.rs`)

**Behavior**:
- Recipe IDs are path-based and WILL CHANGE when file is renamed
- IDs are stable across content edits (same git_path = same ID)
- Renames only occur when recipe title in metadata changes (rare operation)

**Client Handling**:
- If bookmarked ID returns 404, use fallback lookup by recipe name
- Provide search endpoints for clients to find recipes by name or path

**Implementation**:
- Keep path-based ID generation (no additional storage)
- Add helper endpoints: `GET /recipes/find-by-name?q=...` and `GET /recipes/find-by-path?path=...`
- Document behavior clearly in API.md
- Clients should treat IDs as temporary identifiers, not permanent references

**Philosophy**:
- Aligns with "git is source of truth" (no external ID metadata)
- Simple and maintainable
- Suitable for self-hosted family scenario where renames are rare
- No migration complexity for existing recipes
