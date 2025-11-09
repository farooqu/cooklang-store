# Task 6.2: Integration Tests

**Status**: ✅ COMPLETE
**Milestone**: category-path-refactor
**Phase**: 6 (Testing & Verification)
**Branch**: feat/category-path-refactor

---

- [x] Create recipe with default path (root) - content must have YAML front matter `title`
- [x] Create recipe with hierarchical path - YAML front matter required
- [x] Create recipe with missing title in YAML front matter - returns 400 Bad Request
- [x] Update recipe: only content (should rename file if title changed)
  - [x] Old title: "Chocolate Cake", new title: "Dark Chocolate Cake" → file renamed
- [x] Update recipe: only path (should move to new directory, keep title/filename)
- [x] Update recipe: both path and content with new title
- [x] Verify file alignment: created file name matches generated name from title
- [x] Test Git history tracks renames correctly (git mv preserves history)
- [x] List/search returns correct recipe_name, path, file_name for all recipes
- [x] Test find-by-name endpoint: exact match and partial match
- [x] Test find-by-name pagination: limit and offset
- [x] Test find-by-path endpoint: finds recipe at exact path
- [x] Test find-by-path with root path (empty string or omitted)
- [x] Test find-by-path returns empty array for non-existent path (no 404)
- [x] **Scenario: ID change on rename**
  - [x] Create recipe with title "Chocolate Cake" → capture ID
  - [x] Update recipe content with new title "Dark Chocolate Cake" → file renamed
  - [x] Verify old ID returns 404
  - [x] Verify find-by-name returns recipe with new ID
  - [x] Verify new ID can be used for subsequent operations
