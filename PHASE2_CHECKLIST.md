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

## Task 2.2: Add Filename Generation & Normalization

- [ ] Create function `generate_filename(title: &str) -> String`
- [ ] Convert title to lowercase, replace spaces/special chars with hyphens
- [ ] Append `.cook` extension
- [ ] Create function to normalize paths (remove leading/trailing slashes, validate allowed chars)
- [ ] Add unit tests (normal names, special chars, unicode, edge cases)

**Commit Message**: `[Phase2.2] Add filename generation and path normalization utilities`

---

## Task 2.3: Handle File Renaming Logic

- [ ] Create function `should_rename_file(old_name: &str, new_title: &str) -> bool`
- [ ] Detect misalignment: generated name ≠ actual file name
- [ ] Return true if names differ (always sync on write operations)
- [ ] Add unit tests for various scenarios

**Commit Message**: `[Phase2.3] Add file renaming detection logic`

---

## Task 2.4: Update Repository Layer

- [ ] Modify `Recipe` struct if needed to ensure `git_path` is always accurate
- [ ] Update `create()` method to: extract title from content, generate filename, create in correct path
- [ ] Update `update()` method to: extract title, detect if rename needed, perform rename in git
- [ ] Ensure `list_all()` populates `recipe_name` correctly (derived from git_path filename)
- [ ] Add tests for create/update operations with file renaming

**Commit Message**: `[Phase2.4] Update repository layer with title extraction and file renaming`

---

## Definition of Done

- [ ] All 4 tasks completed
- [ ] Unit tests pass for metadata extraction, filename generation, and rename detection
- [ ] Integration tests pass for create/update with file renaming
- [ ] `cargo clippy` and `cargo fmt` pass
- [ ] Code coverage maintained (>80% on new functions)
- [ ] PHASE2_CHECKLIST.md deleted (work complete)
