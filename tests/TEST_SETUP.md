# Test Setup Guide

This document explains how the test infrastructure works and how to use fixture files in integration tests.

## Overview

The test suite uses fixture files (pre-written Cooklang recipes) stored in `tests/fixtures/` that are used in API requests. The tests verify both the API behavior and the side effects (files created on disk, git commits made, etc.).

## Test Structure

There are three main test files:

1. **api_integration_tests.rs** - Tests the REST API with git storage backend
2. **disk_storage_tests.rs** - Tests the REST API with disk storage backend
3. **git_storage_tests.rs** - Tests the REST API with git storage backend, includes git-specific verification

All use the `common.rs` module for shared setup and helper functions.

## How Test Setup Works

### Option 1: Empty Repository (for CREATE tests)

```rust
let (build_router, temp_dir) = setup_api_with_storage("git").await;
let app = build_router();
```

This creates an empty temporary directory that serves as the data directory for the test. Use this when testing CREATE/UPDATE/DELETE operations.

### Option 2: Pre-seeded Repository (for GET/LIST/SEARCH tests)

```rust
let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
    "git",
    vec![
        ("recipe-1", Some("desserts"), "recipe-1.cook"),
        ("recipe-2", Some("desserts"), "recipe-2.cook"),
    ],
).await;

let app = build_router();
```

This:
1. Creates a temporary directory
2. Seeds the specified fixture files into the correct directory structure
3. Initializes the repository (cache loads the seeded files)
4. Returns the router builder

For nested categories:

```rust
let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
    "git",
    vec![
        ("thai-green-curry", Some("meals/asian/thai"), "thai-green-curry.cook"),
    ],
).await;
```

This seeds the fixture into `temp_dir/recipes/meals/asian/thai/thai-green-curry.cook`.

### Manual Fixture Seeding (Advanced)

If you need to seed files after repository creation, use `copy_fixture_to_recipes_dir()`:

```rust
let (build_router, temp_dir) = setup_api_with_storage("git").await;

copy_fixture_to_recipes_dir(
    &temp_dir,
    "test-recipe",           // fixture name (without .cook)
    Some("desserts"),        // category (optional)
    "test-recipe.cook"       // filename to create
);
```

**Note**: This approach doesn't re-initialize the repository, so the cache won't pick up the new files. Only use this for manual file verification.

### Using Fixtures in API Requests

Load fixture content for making API calls:

```rust
let content = load_recipe_fixture("test-recipe");
let payload = serde_json::json!({
    "name": "Test Recipe",
    "content": content,
    "category": "desserts"
});

let response = app.oneshot(make_request("POST", "/api/v1/recipes", Some(payload))).await.unwrap();
```

### Verifying Results on Disk

Use helper functions to verify files were created or deleted:

```rust
// Verify file exists
let filename = verify_recipe_file_exists(&temp_dir, "Test Recipe", "desserts");

// Read file contents
let contents = read_recipe_file(&temp_dir, "Test Recipe", "desserts");
assert_eq!(contents, content);

// Verify file was deleted
verify_recipe_file_deleted(&temp_dir, "Test Recipe", "desserts");
```

For git storage, also verify commits:

```rust
let commit_count = count_git_commits(&temp_dir);
assert!(commit_count > 0);
```

## Fixture Files Reference

Fixture files are located in `tests/fixtures/` and include:

- **Basic recipes**: test-recipe, chocolate-cake, vanilla-cake, cake, pasta, etc.
- **Update operations**: original-name, updated-name (test rename operations)
- **Nested categories**: chicken-biryani, thai-green-curry, pad-thai, etc.
- **Lifecycle**: to-delete (for deletion tests)

See `tests/fixtures/README.md` for the complete inventory.

## Test Patterns

### CREATE/UPDATE/DELETE Tests
These tests use `setup_api_with_storage()` to create an empty repository, then:
1. Make API requests that create/modify/delete files
2. Verify results on disk using helper functions

Example:
```rust
let (build_router, temp_dir) = setup_api_with_storage("git").await;
let app = build_router();

let content = load_recipe_fixture("test-recipe");
let payload = serde_json::json!({
    "name": "Test Recipe",
    "content": content.clone(),
    "category": "desserts"
});

app.oneshot(make_request("POST", "/api/v1/recipes", Some(payload))).await.unwrap();

// Verify file was created
let filename = verify_recipe_file_exists(&temp_dir, "Test Recipe", "desserts");
let contents = read_recipe_file(&temp_dir, "Test Recipe", "desserts");
assert_eq!(contents, content);
```

### GET/LIST/SEARCH Tests
These tests use `setup_api_with_seeded_fixtures()` to pre-populate the repository, then:
1. Make GET/LIST/SEARCH requests
2. Verify the responses include the pre-seeded data

Example:
```rust
let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
    "git",
    vec![
        ("chocolate-cake", Some("desserts"), "chocolate-cake.cook"),
        ("vanilla-cake", Some("desserts"), "vanilla-cake.cook"),
    ],
).await;

let app = build_router();
let response = app.oneshot(make_request("GET", "/api/v1/recipes", None)).await.unwrap();

// Verify we got the pre-seeded recipes
let json: Value = serde_json::from_str(&extract_response_body(response).await).unwrap();
assert_eq!(json["recipes"].as_array().unwrap().len(), 2);
```

**Benefits of this pattern**:
- **GET tests are isolated** from CREATE logic
- **Focused testing** - each test type focuses on its responsibility
- **Independent failures** - if CREATE breaks, GET tests still work
- **Realistic scenarios** - tests pre-populated cache like production

## Common Helper Functions

| Function | Purpose |
|----------|---------|
| `load_recipe_fixture(name)` | Read fixture file content into a string |
| `copy_fixture_to_recipes_dir(temp_dir, fixture_name, category, filename)` | Copy fixture file to temp dir |
| `setup_api_with_storage(storage_type)` | Create temp dir and API router |
| `make_request(method, uri, body)` | Build an HTTP request |
| `extract_response_body(response)` | Extract string body from response |
| `verify_recipe_file_exists(temp_dir, name, category)` | Assert file exists on disk |
| `read_recipe_file(temp_dir, name, category)` | Read created file from disk |
| `verify_recipe_file_deleted(temp_dir, name, category)` | Assert file was deleted |
| `count_git_commits(temp_dir)` | Count commits in git repo |

## Complete Examples

### Example: GET Test with Pre-seeded Data
```rust
#[tokio::test]
async fn test_list_recipes_with_pagination() {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        "git",
        vec![
            ("recipe-1", Some("desserts"), "recipe-1.cook"),
            ("recipe-2", Some("desserts"), "recipe-2.cook"),
        ],
    )
    .await;

    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipes"].as_array().unwrap().len(), 2);
    assert_eq!(json["pagination"]["total"], 2);
}
```

### Example: CREATE Test
```rust
#[tokio::test]
async fn test_create_recipe() {
    let (build_router, temp_dir) = setup_api_with_storage("git").await;
    let app = build_router();

    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": content.clone(),
        "category": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    // Verify file was created on disk
    let filename = verify_recipe_file_exists(&temp_dir, "Test Recipe", "desserts");
    assert!(filename.ends_with(".cook"));

    // Verify file contents
    let contents = read_recipe_file(&temp_dir, "Test Recipe", "desserts");
    assert_eq!(contents, content);
}
```
