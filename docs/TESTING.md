# Testing Guide

## Overview

Cooklang Store includes comprehensive testing at multiple levels:

1. **Rust Integration Tests** - Full API testing in development environment
2. **Docker Integration Tests** - Deployment-level smoke tests
3. **Unit Tests** - Modular business logic tests

## Running Tests

### Prerequisites

For Rust tests, you need:
- Rust 1.83+ ([install from rustup.rs](https://rustup.rs))
- Cargo (comes with Rust)

For Docker tests, you only need:
- Docker (any recent version)

### Rust Integration Tests

Run all tests:
```bash
cargo test
```

Run disk storage tests only:
```bash
cargo test --test disk_storage_tests
```

Run git storage tests only:
```bash
cargo test --test git_storage_tests
```

Run specific test:
```bash
cargo test test_create_recipe
```

Run with output:
```bash
cargo test -- --nocapture
```

Run a specific test module:
```bash
cargo test --test disk_storage_tests
```

## Test Coverage

### Integration Tests (72 total test cases)

Tests are organized into dedicated modules for each storage backend, ensuring comprehensive coverage:

- **`tests/disk_storage_tests.rs`** (24 tests) - Tests API with DiskStorage backend
- **`tests/git_storage_tests.rs`** (24 tests) - Tests API with GitStorage backend + git verification
- **`tests/api_integration_tests.rs`** (24 tests) - Legacy integration tests using default storage (disk)
- **`tests/common.rs`** (shared helpers) - Common test utilities and git verification helpers

**Key Feature**: Git storage tests verify both API responses AND git repository state (files created/updated/deleted at correct paths with correct contents)

**Note**: The legacy `api_integration_tests.rs` is kept for backward compatibility. New tests should use `disk_storage_tests.rs` or `git_storage_tests.rs` depending on the backend being tested.

#### Health & Status (2 tests)
- `test_health_check` - Verify `/health` endpoint responds with 200
- `test_status_endpoint` - Verify `/api/v1/status` returns server metrics

#### Recipe Creation (5 tests)
- `test_create_recipe` - Create a recipe and verify response
- `test_create_recipe_with_comment` - Create recipe with optional comment field for git commit message
- `test_create_recipe_empty_name` - Validation: empty name returns 400
- `test_create_recipe_empty_category` - Validation: behavior differs by backend
  - DiskStorage: accepts (treats as uncategorized)
  - GitStorage: rejects with 400 (requires category for proper git path structure)
- `test_create_recipe_empty_content` - Validation: empty content returns 400

#### Recipe Retrieval (4 tests)
- `test_list_recipes_empty` - List returns empty array when no recipes
- `test_list_recipes_with_pagination` - Create multiple recipes and list them
- `test_list_recipes_with_limit` - Test pagination with custom limit parameter
- `test_get_recipe_by_id` - Create recipe and retrieve by recipe_id
- `test_get_recipe_not_found` - Non-existent recipe returns 404

#### Search (3 tests)
- `test_search_recipes_empty` - Search returns empty results when no matches
- `test_search_recipes_by_name` - Search finds recipes by partial name match
- `test_search_case_insensitive` - Search is case-insensitive

#### Categories (4 tests)
- `test_list_categories_empty` - List returns empty array initially
- `test_list_categories` - Create recipes in different categories and list
- `test_get_recipes_in_category` - Get recipes filtered by category
- `test_get_category_not_found` - Non-existent category returns 404

#### Updates (2 tests)
- `test_update_recipe` - Update recipe name, content, and category; verify file moved/updated in git repo
- `test_update_recipe_not_found` - Updating non-existent recipe returns 404

#### Deletion (2 tests)
- `test_delete_recipe` - Delete recipe and verify it's removed from git repo filesystem
- `test_delete_recipe_not_found` - Deleting non-existent recipe returns 404

#### Status Tracking (1 test)
- `test_status_updates_with_recipes` - Verify status endpoint updates recipe/category counts

#### Git Repository Verification Helpers

Integration tests include helper functions to verify git repository state:

- `verify_recipe_file_exists(temp_dir, name, category)` - Assert recipe file exists at expected path
  - Handles name-to-slug conversion
  - Handles duplicate naming (e.g., recipe-name-2.cook)
  - Returns filename for further inspection

- `read_recipe_file(temp_dir, name, category)` - Read and return recipe file contents
  - Verifies file exists and is readable
  - Used to verify content matches input

- `count_git_commits(temp_dir)` - Count git commits in the test repository
  - Verifies that operations are being recorded in git history
  - Used to ensure commits are being created

- `verify_recipe_file_deleted(temp_dir, name, category)` - Assert recipe file no longer exists
  - Verifies file is removed after deletion
  - Handles cases where entire category directory may be removed

**Example Integration Test with Git Verification**:

```rust
#[tokio::test]
async fn test_create_recipe() {
    let (build_router, temp_dir) = setup_api().await;
    let app = build_router();

    let payload = serde_json::json!({
        "name": "Chocolate Cake",
        "content": "# Recipe\n\n@flour{2%cup}",
        "category": "desserts"
    });

    // Create via API
    let response = app.oneshot(
        make_request("POST", "/api/v1/recipes", Some(payload.clone()))
    ).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    // Verify file created in git repo
    verify_recipe_file_exists(&temp_dir, "Chocolate Cake", "desserts");
    
    // Verify file contents
    let contents = read_recipe_file(&temp_dir, "Chocolate Cake", "desserts");
    assert_eq!(contents, payload["content"]);
    
    // Verify git commit recorded
    assert!(count_git_commits(&temp_dir) > 0);
}
```

### Docker Integration Tests

Located in `scripts/docker-test.sh`

Tests the built Docker image without requiring Rust:

- `/health` - Health check
- `/api/v1/status` - Server status
- `/api/v1/categories` - List categories
- `/api/v1/recipes` - List recipes
- `/api/v1/recipes/search` - Search recipes
- `/api/v1/recipes` - Create recipe

Run tests:
```bash
./scripts/docker-test.sh
```

See [docs/DOCKER-TESTING.md](DOCKER-TESTING.md) for more details.

## Test Data

### Rust Integration Tests

Tests create recipes on-the-fly:
- Sample recipes with various content (ingredients, cookware, etc.)
- Multiple categories (desserts, main, appetizers)
- Recipes with and without optional fields

Tests use `tempfile::TempDir` for isolated git repositories.

### Docker Integration Tests

Initializes `/tmp/cooklang-test-recipes-$$` with:
- Git repository with user `test@example.com`
- Sample chocolate cake recipe in `recipes/desserts/`
- Initial commit with sample data

Repository is cleaned up after tests complete.

## Test Assertions

### HTTP Status Codes

| Scenario | Code |
|----------|------|
| Successful GET | 200 |
| Successful POST (create) | 201 |
| Successful DELETE | 204 |
| Invalid input | 400 |
| Not found | 404 |

### Response Structure

All JSON responses follow the structure defined in `src/api/responses.rs`:

**Create Recipe Response (201):**
```json
{
  "recipe_id": "abc123def456",
  "name": "Recipe Name",
  "description": "Optional description",
  "category": "desserts",
  "content": "Full recipe content"
}
```

**List Recipes Response (200):**
```json
{
  "recipes": [
    {
      "recipe_id": "abc123",
      "name": "Recipe 1",
      "description": "...",
      "category": "desserts"
    }
  ],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "total": 1
  }
}
```

**Status Response (200):**
```json
{
  "status": "running",
  "version": "0.1.0",
  "recipe_count": 5,
  "categories": 3
}
```

## Debugging Tests

### Verbose Test Output

```bash
cargo test -- --nocapture
```

### Run Single Test with Output

```bash
cargo test test_create_recipe -- --nocapture
```

### Run Tests in Single-Threaded Mode

```bash
cargo test -- --test-threads=1
```

This prevents race conditions when debugging.

## Continuous Integration

Tests are designed to run in CI/CD pipelines:

1. Rust tests run during development/PR
2. Docker tests run before deployment
3. Both can run in parallel

Example GitHub Actions workflow:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.83
      - run: cargo test

  docker-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: ./scripts/docker-test.sh
```

## Adding New Tests

When adding a new feature:

1. **Write Rust integration test first** (TDD)
2. Test the happy path
3. Test validation/error cases
4. Test edge cases
5. Run: `cargo test`
6. Implement feature
7. Verify all tests pass
8. Update Docker tests if needed (smoke test only)

Example test structure:

```rust
#[tokio::test]
async fn test_feature_name() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    // Arrange
    let payload = serde_json::json!({...});

    // Act
    let response = app.oneshot(
        make_request("POST", "/api/v1/endpoint", Some(payload))
    ).await.unwrap();

    // Assert
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["field"], "expected_value");
}
```

## Coverage Target

- **Goal**: >80% code coverage
- **Method**: Combination of unit tests and integration tests
- **Measurement**: Run `cargo tarpaulin` (after installing)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

This generates a coverage report in `tarpaulin-report.html`.
