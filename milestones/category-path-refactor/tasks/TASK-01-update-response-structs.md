# TASK: Phase 3.1 - Update Response Structs

**Status**: ❌ NOT STARTED
**Milestone**: category-path-refactor
**Phase**: 3 (API Response Layer)
**Task**: 3.1

---

## Overview

Update API response structs (`RecipeResponse` and `RecipeSummary`) in `src/api/responses.rs` to:
- Use camelCase JSON field names via serde attributes
- Add new fields (`path`, `fileName` for full response)
- Rename existing fields (`name` → `recipe_name`, etc.)
- Properly handle optional fields (null omission)

---

## RecipeResponse Updates

**File**: `src/api/responses.rs`

- [ ] Rename `name` field to `recipe_name` with `#[serde(rename = "recipeName")]`
- [ ] Add `path: Option<String>` field
- [ ] Add `file_name: String` field with `#[serde(rename = "fileName")]`
- [ ] Keep `description: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`
- [ ] Keep `content: String`
- [ ] Rename `id` to `recipe_id` with `#[serde(rename = "recipeId")]` (if not already done)
- [ ] Verify serialization with unit tests
- [ ] Example output:
  ```json
  {
    "recipeId": "a1b2c3d4e5f6",
    "recipeName": "Chocolate Cake",
    "path": "desserts",
    "fileName": "chocolate-cake.cook",
    "content": "...",
    "description": null
  }
  ```

---

## RecipeSummary Updates

**File**: `src/api/responses.rs`

- [ ] Rename `name` field to `recipe_name` with `#[serde(rename = "recipeName")]`
- [ ] Add `path: Option<String>` field
- [ ] Remove `file_name` field (not in summaries)
- [ ] Remove `description` field (not in summaries)
- [ ] Rename `id` to `recipe_id` with `#[serde(rename = "recipeId")]` (if not already done)
- [ ] Keep only: `recipe_id`, `recipe_name`, `path`
- [ ] Verify serialization with unit tests
- [ ] Example output:
  ```json
  {
    "recipeId": "a1b2c3d4e5f6",
    "recipeName": "Chocolate Cake",
    "path": "desserts"
  }
  ```

---

## Testing

- [ ] Add/update unit tests for `RecipeResponse` serialization
- [ ] Add/update unit tests for `RecipeSummary` serialization
- [ ] Test that null fields are omitted from JSON output
- [ ] Test that non-null fields are present in JSON output
- [ ] Verify camelCase field names in serialized JSON

---

## Verification

- [ ] `cargo build` compiles without errors
- [ ] `cargo test` passes all tests
- [ ] `cargo clippy` shows no new warnings
- [ ] Serialized JSON matches expected format with camelCase fields

---

## Definition of Done

- [x] RecipeResponse struct updated with all new fields and serde attributes
- [x] RecipeSummary struct updated with correct fields and serde attributes
- [x] All response tests passing
- [x] Serialization produces camelCase JSON output
- [x] Optional fields properly omitted when null
