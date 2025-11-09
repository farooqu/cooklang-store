# Milestone: Category Field Semantics & Path Handling Refactor

**Status**: Phases 1-4 Complete | Nov 9, 2025

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

## Phase 3: API Response Layer ✅ COMPLETE

**Completed** (Nov 9, 2025):
- Updated RecipeResponse with camelCase fields: recipeId, recipeName, path, fileName, content, description
- Updated RecipeSummary with camelCase fields: recipeId, recipeName, path
- Removed deprecated `name` and `category` fields from response structs
- Updated CreateRecipeRequest and UpdateRecipeRequest with correct field sets
- All serde attributes properly configured for camelCase JSON serialization

---

## Phase 4: API Handlers ✅ COMPLETE

**Completed** (Nov 9, 2025):
- Updated all CRUD handlers (create, update, get, list) with new request/response schemas
- Validation enforces YAML front matter with `title` field for content
- Title extraction from metadata, filename generation, and file renaming all working
- Added two fallback lookup endpoints for ID migration: find-by-name and find-by-path
- All 70 API integration tests passing

---

## Phase 5: Documentation & Examples ❌ NOT STARTED

See: `milestones/category-path-refactor/phases/05-documentation.md`

---

## Phase 6: Testing & Verification ❌ NOT STARTED

See: `milestones/category-path-refactor/phases/06-testing-verification.md`

---

## Definition of Done

- [ ] All API spec files updated (openapi.yaml, API.md, postman-collection.json)
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
