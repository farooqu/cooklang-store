# Task 5.2: Remove SAMPLE-RECIPES and Reference Test Fixtures

**Status**: ⏳ IN PROGRESS
**Milestone**: category-path-refactor
**Phase**: 5 (Documentation & Examples)
**Branch**: feat/category-path-refactor

---

## Subtasks

- [ ] Delete docs/SAMPLE-RECIPES.md (redundant with test fixtures)
- [ ] Review test fixtures in tests/fixtures/
  - [ ] Verify all fixtures have YAML front matter with `title` field
  - [ ] Check fixtures are syntactically valid
- [ ] Update API.md to remove sample recipes section and instead reference test fixtures
  - [ ] Add note: "For testing, use fixtures from tests/fixtures/"
  - [ ] Update curl examples to use test fixture content (where applicable)
- [ ] Update README.md if it references SAMPLE-RECIPES.md
  - [ ] Remove references to SAMPLE-RECIPES.md
  - [ ] Point to tests/fixtures/ for recipe examples
- [ ] Update tests/TEST_SETUP.md or tests/fixtures/README.md with guidance:
  - [ ] Explain how fixtures are used in tests
  - [ ] Clarify fixture structure for API examples
- [ ] Commit changes with message: "[category-path-refactor] Task 5.2 - Remove sample recipes, use test fixtures"

---

## Notes

- Test fixtures location: `tests/fixtures/` (contains 20+ .cook files)
- All recipe content must include YAML front matter with `title` field
- Test fixtures serve dual purpose: unit tests + API documentation examples
- Keep docs focused on API usage, not recipe content sampling
