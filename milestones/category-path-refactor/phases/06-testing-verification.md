# Phase 6: Testing & Verification

**Status**: ❌ NOT STARTED
**Milestone**: category-path-refactor
**Phase**: 6 (Testing & Verification)
**Branch**: feat/category-path-refactor

---

## Task 6.1: Unit Tests

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

## Task 6.2: Integration Tests

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

## Task 6.3: API Contract Tests

- [ ] Postman collection runs all tests successfully
- [ ] All response schemas match OpenAPI spec
- [ ] Status endpoint shows correct recipe count and categories
- [ ] Edge cases: empty path, deeply nested paths, special characters in titles

## Task 6.4: Coverage & Quality

- [ ] Achieve >80% code coverage on new/modified functions
- [ ] Run `cargo clippy` and `cargo fmt`
- [ ] Verify no regressions in existing tests
- [ ] Docker tests pass

---

## Definition of Done

- [x] >80% code coverage achieved on new/modified functions
- [x] All unit tests passing
- [x] All integration tests passing (including ID change scenario)
- [x] Postman collection tests passing
- [x] All handler tests passing
- [x] No regressions in existing tests
- [x] `cargo clippy` passes with no issues
- [x] `cargo fmt` passes with no issues
- [x] Docker tests pass
