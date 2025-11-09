# Phase 1: API Specification

**Status**: ✅ COMPLETE
**Milestone**: category-path-refactor
**Completed**: Nov 9, 2025

## Task 1.1: Update OpenAPI Spec (openapi.yaml)

### RecipeResponse schema
- [x] Update to include: `recipeId`, `recipeName`, `path`, `fileName`, `description`, `content`
- [x] Mark required: `recipeId`, `recipeName`, `content`, `fileName`
- [x] Mark optional: `path`, `description` (nullable)
- [x] Add `x-nullable: true` or use nullable for `description` and `path`

### RecipeSummary schema
- [x] Update to include: `recipeId`, `recipeName`, `path`
- [x] Remove: `fileName`, `content`, `description`
- [x] Mark required: `recipeId`, `recipeName`
- [x] Mark optional: `path`

### CreateRecipeRequest schema
- [x] Keep only: `content` (required), `path` (optional), `author` (optional), `comment` (optional)
- [x] Remove: `name`, `category`

### UpdateRecipeRequest schema
- [x] Keep only: `content` (optional), `path` (optional), `author` (optional), `comment` (optional)
- [x] Add validation note: at least one of `content` or `path` required
- [x] Remove: `name`, `category`

### Fallback Endpoints (new)
- [x] Add `GET /api/v1/recipes/find-by-name` operation
  - [x] Query param: `q` (search term, required)
  - [x] Query params: `limit`, `offset` (pagination, optional)
  - [x] Response: array of `RecipeSummary`
- [x] Add `GET /api/v1/recipes/find-by-path` operation
  - [x] Query param: `path` (recipe path, required)
  - [x] Response: single `RecipeSummary`
  - [x] 404 if path not found

### Misc
- [x] Update all endpoint response examples to use new RecipeResponse/RecipeSummary format
- [x] Update all request examples to match new payloads
- [x] Add note about Recipe ID stability in spec description or info section
- [x] Validate YAML syntax: `python3 -c "import yaml; yaml.safe_load(open('docs/openapi.yaml'))"`

## Task 1.2: Update API.md Documentation
- [x] Update Common Response Format section with both RecipeResponse and RecipeSummary schemas
- [x] Add note: null fields omitted from JSON (using `skip_serializing_if`)
- [x] Update Create Recipe endpoint: show request with only `content` and optional `path`, `author`, `comment`
- [x] Update Create Recipe response: full RecipeResponse format with all fields
- [x] Update List Recipes endpoint: response is RecipeSummary (no fileName, no content, no description)
- [x] Update Get Single Recipe endpoint: full RecipeResponse with all fields and fileName
- [x] Update Update Recipe endpoint: request with only optional `content` and `path`
- [x] Add new section: "Fallback Lookup Endpoints"
  - [x] Document `GET /recipes/find-by-name` with examples
  - [x] Document `GET /recipes/find-by-path` with examples
  - [x] Explain use case: recovering from recipe ID changes due to renames
- [x] Add note about recipe names being derived from Cooklang YAML front matter (`title` field)
- [x] Add examples showing YAML front matter format (with `---` delimiters) and how titles map to filenames
- [x] Add examples showing how file names are generated from recipe titles (normalization rules)
- [x] Add note about file renaming behavior on updates (happens automatically on write)
- [x] Add section: "Recipe ID Stability" explaining that IDs change on rename, clients should use lookup endpoints as fallback
- [x] Update all curl examples to match new request format with proper front matter

---

## Progress Notes

**Started**: Nov 9, 2025
**Branch**: feat/category-path-refactor-phase1

### Next Steps
1. Review existing OpenAPI spec to understand current structure
2. Update RecipeResponse, RecipeSummary, CreateRecipeRequest, UpdateRecipeRequest schemas
3. Add fallback endpoint specs
4. Update API.md documentation
5. Validate changes
