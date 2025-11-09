# Task: Update & Extend Docker Smoke Tests for Phase 2 Path Handling Changes

## Context

Phase 2.4 has introduced significant changes to how recipes are stored and renamed based on their YAML front matter titles:

- **Title Extraction**: Recipe titles are now extracted from YAML front matter `title` field
- **Filename Generation**: Filenames are generated from extracted titles (lowercase, hyphenated)
- **File Renaming**: When a recipe's title changes, the file is renamed and git_path changes
- **Recipe ID Changes**: Since recipe_id is derived from git_path (SHA256 of git_path), IDs change on rename
- **Nested Categories**: Support for hierarchical category paths (e.g., `meals/meat/traditional`)

## Objective

Update Docker smoke tests to verify these new behaviors work correctly end-to-end in a containerized environment.

## What Needs to Be Done

### 1. Identify Broken Smoke Tests
- Run existing Docker smoke tests: `scripts/docker-test.sh`
- Document which tests fail and why (likely due to hardcoded filename/path assumptions)
- Check `docs/DOCKER-TESTING.md` for test documentation

### 2. Fix Existing Tests
Update failing tests to work with new path/naming behavior:
- Adjust fixture file paths if needed
- Update expected filenames in assertions (now derived from YAML title, not request name)
- Fix any hardcoded recipe_id expectations
- Handle path changes when titles change

### 3. Add New Tests for Nested Categories
Create tests to verify hierarchical category support:
- **Test: Create recipe in nested category**
  - Create recipe with category `meals/meat/traditional`
  - Verify file created at `recipes/meals/meat/traditional/filename.cook`
  - Verify category extraction works correctly

- **Test: Move recipe between nested categories**
  - Create recipe in `desserts/cakes`
  - Move to `meals/meat/traditional`
  - Verify old path removed, new path correct

- **Test: Search across nested categories**
  - Seed multiple recipes in different nested paths
  - Search by name, verify results
  - Filter by category, verify nested categories work

### 4. Add New Tests for YAML-Driven Filenames
Create tests verifying title extraction and file renaming:
- **Test: File rename on title update**
  - Create recipe with title "Chocolate Cake" → file `chocolate-cake.cook`
  - Update to title "Dark Chocolate Cake" → file should be `dark-chocolate-cake.cook`
  - Verify old filename deleted, new filename exists

- **Test: Title extraction from YAML**
  - Create recipe with `title: Complex Title!` in YAML
  - Verify filename is `complex-title.cook` (special chars removed)
  - Verify recipe name matches YAML title, not request name

- **Test: Missing YAML title fails**
  - POST recipe without YAML front matter title
  - Verify 400 Bad Request response

## How to Approach

### Setup
```bash
cd /workspaces/cooklang-store
# Ensure Docker is running
docker ps  # Should list running containers
```

### Testing Strategy
1. **Run existing tests first**: `bash scripts/docker-test.sh`
2. **Identify failures**: Note which tests fail and error messages
3. **Update fixtures if needed**:
   - Check `tests/fixtures/` for fixture files
   - Consider what category/path each fixture should be copied to
   - Update `common.rs` fixture copying logic if needed
4. **Create test script**: Add new test cases to `tests/api_integration_tests.rs` 
5. **Verify in Docker**: Run full Docker test before submitting

### Fixture Strategy
Pre-seed fixtures into the Docker container at specific paths to test nested category handling:

Example fixture mappings to add:
```
("chocolate-cake", Some("desserts/cakes"), "chocolate-cake.cook")
("tiramisu", Some("desserts/italian"), "tiramisu.cook")
("chicken-biryani", Some("meals/meat/traditional"), "chicken-biryani.cook")
("pasta", Some("italian/pasta"), "pasta.cook")
```

Use these in new integration tests with `setup_api_with_seeded_fixtures()`.

## Files to Modify

1. **tests/api_integration_tests.rs**
   - Fix broken tests to match new behavior
   - Add new tests for nested categories
   - Add new tests for YAML-driven renaming

2. **tests/common.rs**
   - Update helper functions if fixture copying logic needs adjustment
   - Add new helper for verifying nested category structure

3. **scripts/docker-test.sh** (if needed)
   - Update test commands if Docker image changes
   - Add new test cases

4. **docs/DOCKER-TESTING.md**
   - Update documentation to reflect new tests
   - Document nested category testing approach

## Success Criteria

- [ ] All existing Docker smoke tests pass (fixed if broken)
- [ ] 3+ new tests added for nested category support
- [ ] 3+ new tests added for YAML-driven filename changes
- [ ] Fixtures properly seeded to different nested paths
- [ ] Docker image builds without errors
- [ ] All tests pass both locally and in Docker
- [ ] Documentation updated with new test cases
- [ ] No regressions in existing test suite

## Important Notes

### Recipe ID Behavior
- Recipe ID = SHA256 of git_path (first 12 hex chars)
- When git_path changes (due to rename), recipe_id changes
- Tests should NOT hardcode recipe_id values, use response values instead
- Old recipe_id should return 404 after rename

### YAML Front Matter Requirements
- ALL recipes MUST have `---\ntitle: Recipe Name\n---\n` at start
- Title field is required (no default fallback in create)
- Only the `title` field matters for filename generation
- Extraction is case-insensitive

### File Naming Rules
- Title "Chocolate Cake" → `chocolate-cake.cook`
- Title "Pad Thai (Thai Noodles)" → `pad-thai-thai-noodles.cook`
- Special chars removed, spaces become hyphens
- Multiple consecutive hyphens collapsed to single hyphen

### Path Structure
- Root recipes: `recipes/filename.cook`
- Single category: `recipes/desserts/filename.cook`
- Nested category: `recipes/meals/meat/traditional/filename.cook`

## Commands You'll Need

```bash
# Run existing smoke tests
bash scripts/docker-test.sh

# Build Docker image
docker-compose build

# Run tests in Docker
docker-compose run --rm app cargo test

# View Docker logs
docker-compose logs -f app

# Interactive bash in container
docker-compose exec app bash

# Full cleanup between runs
docker-compose down -v
docker system prune -f
```

## Next Steps After Completion

1. Commit with message: `[Phase2.4] Update and extend Docker smoke tests for nested categories and file renaming`
2. Document any issues encountered in `docs/DOCKER-TESTING.md`
3. Update `PROJECT_PLAN.md` if Docker-related work is identified for future phases
