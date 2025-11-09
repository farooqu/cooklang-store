# Task 6.1: Unit Tests

**Status**: ✅ COMPLETE
**Milestone**: category-path-refactor
**Phase**: 6 (Testing & Verification)
**Branch**: feat/category-path-refactor

---

- [x] Test title extraction from Cooklang YAML front matter (various formats)
  - [x] Standard format:
    ```
    ---
    title: Recipe Name
    ---
    ```
  - [x] Case-insensitive key lookup (Title, TITLE, etc.)
  - [x] Missing title returns error
  - [x] Malformed YAML front matter handling (missing `---`, invalid YAML)
- [x] Test filename generation (normal, special chars, unicode, edge cases)
- [x] Test path normalization
- [x] Test file renaming detection logic
- [x] Verify misaligned files are renamed on update
- [x] Validate Cooklang content validation (must have front matter title)
