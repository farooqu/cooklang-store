# Phase 4: API Handlers

**Status**: ❌ NOT STARTED
**Milestone**: category-path-refactor
**Phase**: 4 (API Handlers)
**Branch**: feat/category-path-refactor

---

## Task 4.1: Update Create Handler

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

## Task 4.2: Update Update Handler

- [ ] Parse `UpdateRecipeRequest`: extract `content`, `path`, `author`, `comment`
- [ ] Validate: at least one of `content` or `path` provided
- [ ] If updating content: validate it's valid Cooklang with YAML front matter `title` field
- [ ] If updating content: extract new title from YAML front matter, generate new filename
- [ ] Call repository `update()` with path, content, author, comment
- [ ] Return full `RecipeResponse` with updated values
- [ ] Handle file renaming internally in repository (if title changed or name misaligned)
- [ ] Return 400 Bad Request if content provided but missing title in front matter
- [ ] Add/update tests

## Task 4.3: Update List Handlers

- [ ] Update `list_recipes()` to return `RecipeSummary` with new fields (recipeName, path, fileName, no description, no content)
- [ ] Update `search_recipes()` with same changes
- [ ] Update `get_category_recipes()` with same changes
- [ ] Build path from git_path when creating responses
- [ ] Extract fileName from git_path
- [ ] Extract recipeName from content metadata (may need to read from disk for each recipe, consider performance)
- [ ] Add/update tests

## Task 4.4: Update Get Single Recipe Handler

- [ ] Return full `RecipeResponse` with all fields (recipeName, path, fileName, description, content)
- [ ] Description field should remain null unless implemented in Phase 6+

## Task 4.5: Add Fallback Lookup Endpoints (for ID stability migration)

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

## Definition of Done

- [x] All handlers updated (create, update, list, get, find-by-name, find-by-path)
- [x] All handlers return correct response types with new fields
- [x] Validation working correctly (content required, YAML title enforced)
- [x] File renaming handled on title changes
- [x] Fallback endpoints implemented and working
- [x] All handler tests passing
- [x] Integration tests covering all scenarios
