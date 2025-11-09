# TASK: Phase 3.2 - Update Request Structs

**Status**: ❌ NOT STARTED
**Milestone**: category-path-refactor
**Phase**: 3 (API Response Layer)
**Task**: 3.2

---

## Overview

Update API request structs (`CreateRecipeRequest` and `UpdateRecipeRequest`) in `src/api/models.rs` to:
- Simplify fields to only what's needed for write operations
- Remove deprecated `name` and `category` fields
- Add validation for required/optional field combinations
- Prepare for handler validation in Phase 4

---

## CreateRecipeRequest Updates

**File**: `src/api/models.rs`

- [ ] Keep `content: String` field (required for create)
- [ ] Keep `path: Option<String>` field (optional, defaults to root)
- [ ] Keep `author: Option<String>` field (optional)
- [ ] Keep `comment: Option<String>` field (optional)
- [ ] Remove `name` field (no longer used; name is derived from YAML title)
- [ ] Remove `category` field (replaced by `path`)
- [ ] Remove any other deprecated fields
- [ ] Add doc comment explaining field behavior:
  ```rust
  /// Create a new recipe
  /// 
  /// - `content`: required, must include YAML front matter with `title` field
  /// - `path`: optional directory path (no `recipes/` prefix, defaults to root)
  /// - `author`: optional git commit author
  /// - `comment`: optional git commit message
  ```

---

## UpdateRecipeRequest Updates

**File**: `src/api/models.rs`

- [ ] Make all fields optional: `content: Option<String>`, `path: Option<String>`, etc.
- [ ] Keep `author: Option<String>` field
- [ ] Keep `comment: Option<String>` field
- [ ] Remove `name` field (not updateable; derived from content metadata)
- [ ] Remove `category` field (replaced by `path`)
- [ ] Remove any other deprecated fields
- [ ] Add doc comment explaining field behavior:
  ```rust
  /// Update an existing recipe
  /// 
  /// At least one of `content` or `path` must be provided
  /// 
  /// - `content`: optional new recipe content (must include YAML front matter with `title` if provided)
  /// - `path`: optional new directory path (no `recipes/` prefix)
  /// - `author`: optional git commit author
  /// - `comment`: optional git commit message
  ```

---

## Validation

- [ ] Add validation: `CreateRecipeRequest` requires `content` to be non-empty
- [ ] Add validation: `UpdateRecipeRequest` requires at least one of `content` or `path`
- [ ] Validation can be in struct (via custom derives or serde validation) or in handlers (Phase 4)
- [ ] Document where validation happens (struct vs handler)
- [ ] Add unit tests for validation

---

## Testing

- [ ] Add/update unit tests for `CreateRecipeRequest` deserialization
- [ ] Add/update unit tests for `UpdateRecipeRequest` deserialization
- [ ] Test that missing required fields fail appropriately
- [ ] Test that deprecated fields are properly removed
- [ ] Test JSON payload parsing with new field combinations
- [ ] Example valid payloads:
  - Create: `{ "content": "---\ntitle: Cake\n---\n...", "path": "desserts" }`
  - Update with content: `{ "content": "---\ntitle: New Title\n---\n..." }`
  - Update with path: `{ "path": "meals/desserts" }`
  - Update with both: `{ "content": "...", "path": "meals/desserts" }`

---

## Verification

- [ ] `cargo build` compiles without errors
- [ ] `cargo test` passes all tests
- [ ] `cargo clippy` shows no new warnings
- [ ] Deprecated fields cannot be used in requests
- [ ] New field combinations work correctly

---

## Definition of Done

- [x] CreateRecipeRequest struct has only required fields
- [x] UpdateRecipeRequest struct has only optional fields
- [x] All deprecated fields removed
- [x] Validation rules in place (content required for create, at least one field for update)
- [x] All request tests passing
- [x] Documentation clear about field requirements
