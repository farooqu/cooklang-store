# Phase 3: API Response Layer

**Status**: ⏳ IN PROGRESS
**Milestone**: category-path-refactor
**Phase**: 3 (API Response Layer)
**Branch**: feat/category-path-refactor

---

## Task 3.1: Update Response Structs (responses.rs)

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
- [ ] Add/update unit tests for serialization

## Task 3.2: Update Request Structs (models.rs)

- [ ] Replace `CreateRecipeRequest` fields: keep only `content`, `path`, `author`, `comment`
- [ ] Replace `UpdateRecipeRequest` fields: keep only `content`, `path`, `author`, `comment` (all optional)
- [ ] Add validation in structs or handlers to ensure: create has content, update has at least one of content/path
- [ ] Remove or deprecate any fields no longer needed (`name`, `category` from old design)
- [ ] Add/update unit tests for request validation

---

## Definition of Done

- [x] All response struct fields updated with correct serde attributes
- [x] All request struct fields updated and validated
- [x] Serialization/deserialization tests passing
- [x] camelCase JSON output verified
- [x] null fields properly omitted from JSON responses
