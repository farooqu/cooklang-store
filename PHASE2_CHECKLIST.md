# Phase 2 Checklist: Core Business Logic

**Milestone**: Category Field Semantics & Path Handling Refactor  
**Phase**: 2 (Core Business Logic)  
**Status**: In Progress

---

## Task 2.1: Add Recipe Metadata Extraction Utility ✅ COMPLETE

- [x] Create new function `extract_recipe_title(content: &str) -> Result<String, Error>`
- [x] Parse Cooklang content to extract title from YAML front matter
- [x] Front matter format: YAML block delimited by `---` at start of file
- [x] Extract `title` field from YAML front matter (case-insensitive key lookup)
- [x] Handle edge cases: empty content, missing title, invalid YAML, malformed front matter
- [x] Add unit tests for title extraction (valid cases, edge cases, missing title)
- [x] Document expected format (inline in function or docs/SAMPLE-RECIPES.md)
- [x] Reference Cooklang spec: https://cooklang.org/docs/spec/

**Implementation Details**:
- Added `extract_recipe_title(content: &str) -> Result<String>` in `src/parser/mod.rs`
- Helper function `extract_title_from_yaml()` for YAML parsing
- 19 comprehensive unit tests covering standard format, quotes, case-insensitivity, edge cases, unicode, and error conditions
- All 74 existing tests still pass; no regressions

**Commit Message**: `[Phase2.1] Add recipe metadata extraction utility - extract_recipe_title()`

---

## Task 2.2: Add Filename Generation & Normalization ✅ COMPLETE

- [x] Create function `generate_filename(title: &str) -> String`
- [x] Convert title to lowercase, replace spaces/special chars with hyphens
- [x] Append `.cook` extension
- [x] Create function to normalize paths (remove leading/trailing slashes, validate allowed chars)
- [x] Add unit tests (normal names, special chars, unicode, edge cases)

**Implementation Details**:
- Added `generate_filename(title: &str) -> String` in `src/parser/mod.rs` - converts title to lowercase, replaces special chars with hyphens, removes consecutive/leading/trailing hyphens, appends `.cook` extension
- Added `normalize_path(path: &str) -> Result<String>` in `src/parser/mod.rs` - removes leading/trailing slashes, validates allowed characters (alphanumeric, hyphens, underscores, dots, slashes)
- 30 comprehensive unit tests covering standard cases, special characters, unicode, edge cases, and error conditions
- All 58 existing tests still pass; no regressions

**Commit Message**: `[Phase2.2] Add filename generation and path normalization utilities`

---

## Task 2.3: Handle File Renaming Logic ✅ COMPLETE

- [x] Create function `should_rename_file(old_name: &str, new_title: &str) -> bool`
- [x] Detect misalignment: generated name ≠ actual file name
- [x] Return true if names differ (always sync on write operations)
- [x] Add unit tests for various scenarios

**Implementation Details**:
- Added `should_rename_file(old_filename: &str, new_title: &str) -> bool` in `src/parser/mod.rs`
- Compares current filename with generated filename from new title
- Returns true if they differ (rename needed), false if they match
- Handles various edge cases: title changes, whitespace variations, special character normalization, unicode
- 19 comprehensive unit tests covering all scenarios
- All 125 existing tests still pass; no regressions

**Commit Message**: `[Phase2.3] Add file renaming detection logic`

---

## Task 2.4: Update Repository Layer ✅ COMPLETE

- [x] Modify `Recipe` struct if needed to ensure `git_path` is always accurate
- [x] Update `create()` method to: extract title from content, generate filename, create in correct path
- [x] Update `update()` method to: extract title, detect if rename needed, perform rename in git
- [x] Ensure `list_all()` populates `recipe_name` correctly (derived from git_path filename)
- [x] Add tests for create/update operations with file renaming

**Implementation Details**:
- `Recipe` struct has `git_path`, `file_name`, `name` (recipe title), `description`, `category`, `content`
- `create()` extracts title from YAML front matter, generates filename, writes to storage
- `update()` detects title changes, renames file if needed, handles path changes
- `rebuild_from_storage()` extracts recipe names from YAML front matter with fallback to path-based names

**Tests to Add/Update**:

### New Unit Tests (in src/repository.rs)
- [x] `test_create_with_default_path_root()` - Create recipe with no path (should be in root)
- [x] `test_create_with_hierarchical_path()` - Create recipe with nested path (e.g., "desserts/chocolate")
- [x] `test_create_missing_title_in_yaml_returns_400()` - Verify error when content lacks YAML title field
- [x] `test_create_generates_filename_from_title()` - Verify filename matches generated name from title
- [x] `test_update_only_content_with_title_change()` - Update content with new title → file should rename
  - Create recipe "Chocolate Cake" → filename "chocolate-cake.cook"
  - Update content to have title "Dark Chocolate Cake" → filename should be "dark-chocolate-cake.cook"
  - Old file should be deleted, new git_path should be different
- [x] `test_update_only_path()` - Move recipe to different directory, keep title/filename
- [x] `test_update_both_content_and_path()` - Update content (new title) AND path simultaneously
- [x] `test_update_detects_file_misalignment()` - If current filename doesn't match generated from title, rename on update
- [x] `test_recipe_id_changes_on_rename()` - Verify recipe_id changes when git_path changes (ID stability scenario)

### Existing Tests to Update (src/repository.rs)
The existing tests already pass because they validate core functionality. Just ensure they still work after changes:
- `test_create_recipe()` - Verifies filename generation from extracted title ✓
- `test_update_rename_only()` - Tests title change detection and rename ✓ 
- `test_update_move_only()` - Tests path-only changes (category) ✓
- All create/update/delete tests - Verify with author/comment handling ✓

### Integration Tests to Update/Add (tests/api_integration_tests.rs)

**Tests needing updates to match new API response format**:
Note: Phase 1 already updated OpenAPI spec. Need to verify integration tests work with old API until Phase 3 response changes are done.

Current integration tests use old API contract:
- `test_create_recipe_impl()` - Expects response with `"name"` field, needs update after Phase 3
- `test_create_recipe_with_comment_impl()` - Same as above
- `test_create_recipe_empty_category_impl()` - Same as above
- All list tests (`test_list_recipes_*`, `test_search_recipes_*`) - Need update after Phase 3

**New integration tests to add**:
- [x] `test_create_recipe_missing_yaml_front_matter()` - POST without YAML front matter → 400 Bad Request
- [x] `test_create_recipe_with_valid_yaml_front_matter()` - POST with valid YAML → extracts title
- [x] `test_update_recipe_title_causes_filename_change()` - Update content with new title → file renamed
- [x] `test_id_change_on_rename_scenario()` - Full scenario showing ID stability behavior:
  - Create recipe with title "Chocolate Cake" → capture ID A
  - Update recipe content with new title "Dark Chocolate Cake" → verify old ID A returns 404
  - Verify find-by-name endpoint returns recipe with new ID B (after Phase 4)
  - Verify subsequent operations use new ID B

**Commit Messages**:
1. ✅ `[Phase2.4] Add unit tests for repository layer title extraction and file renaming` 
   - 9 comprehensive unit tests covering create/update operations, file renaming, recipe ID changes
   - All 134 unit tests passing
2. ✅ `[Phase2.4] Add integration tests for YAML front matter validation and file renaming behavior`
   - 8 new integration tests (4 feature tests × 2 backends: git & disk)
   - test_create_recipe_missing_yaml_front_matter: Validates YAML requirement
   - test_create_recipe_with_valid_yaml_front_matter: Validates title extraction
   - test_update_recipe_title_causes_filename_change: Validates file rename on title change
   - test_id_change_on_rename_scenario: Validates recipe_id change and old ID invalidation
   - All 70 integration tests passing
   - Total test suite: 208 tests passing (134 unit + 70 integration + 4 doc)

---

## Definition of Done

- [ ] All 4 tasks completed
- [ ] Unit tests pass for metadata extraction, filename generation, and rename detection
- [ ] Integration tests pass for create/update with file renaming
- [ ] `cargo clippy` and `cargo fmt` pass
- [ ] Code coverage maintained (>80% on new functions)
- [ ] PHASE2_CHECKLIST.md deleted (work complete)
