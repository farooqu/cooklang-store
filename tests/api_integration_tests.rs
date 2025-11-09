mod common;

use common::*;
use serde_json::Value;
use tempfile::TempDir;
use tower::util::ServiceExt;

// ============================================================================
// HEALTH & STATUS TESTS
// ============================================================================

async fn test_health_check_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/health", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_health_check_git() {
    test_health_check_impl("git").await;
}

#[tokio::test]
async fn test_health_check_disk() {
    test_health_check_impl("disk").await;
}

async fn test_status_endpoint_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/status", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["status"], "running");
    assert!(json["version"].is_string());
    assert_eq!(json["recipe_count"], 0);
    assert_eq!(json["categories"], 0);
}

#[tokio::test]
async fn test_status_endpoint_git() {
    test_status_endpoint_impl("git").await;
}

#[tokio::test]
async fn test_status_endpoint_disk() {
    test_status_endpoint_impl("disk").await;
}

// ============================================================================
// RECIPE CREATION TESTS
// ============================================================================

async fn test_create_recipe_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "content": content.clone(),
        "path": "desserts"
    });

    let response = app
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(payload.clone()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipeName"], "Test Recipe");
    assert_eq!(json["path"], "desserts");
    assert!(json["recipeId"].is_string());

    // Verify file was created on disk
    let filename = verify_recipe_file_exists(&temp_dir, "Test Recipe", "desserts");
    assert!(filename.ends_with(".cook"));

    // Verify file contents
    let contents = read_recipe_file(&temp_dir, "Test Recipe", "desserts");
    assert_eq!(contents, content);

    temp_dir
}

#[tokio::test]
async fn test_create_recipe_git() {
    let temp_dir = test_create_recipe_impl("git").await;

    // Git-specific verification: commit was made
    let commit_count = count_git_commits(&temp_dir);
    assert!(commit_count > 0, "Expected at least one commit in git repo");
}

#[tokio::test]
async fn test_create_recipe_disk() {
    let _temp_dir = test_create_recipe_impl("disk").await;
}

async fn test_create_recipe_with_comment_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let content = load_recipe_fixture("chocolate-cake");
    let payload = serde_json::json!({
        "content": content,
        "path": "desserts",
        "comment": "Classic chocolate recipe"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipeName"], "Chocolate Cake");
    assert_eq!(json["path"], "desserts");
}

#[tokio::test]
async fn test_create_recipe_with_comment_git() {
    test_create_recipe_with_comment_impl("git").await;
}

#[tokio::test]
async fn test_create_recipe_with_comment_disk() {
    test_create_recipe_with_comment_impl("disk").await;
}

async fn test_create_recipe_empty_name_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    // Empty name is no longer relevant - test empty content instead (which will also lack YAML front matter)
    let payload = serde_json::json!({
        "content": "",
        "path": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_recipe_empty_name_git() {
    test_create_recipe_empty_name_impl("git").await;
}

#[tokio::test]
async fn test_create_recipe_empty_name_disk() {
    test_create_recipe_empty_name_impl("disk").await;
}

async fn test_create_recipe_empty_category_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "content": content.clone(),
        "path": ""
    });

    let response = app
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(payload.clone()),
        ))
        .await
        .unwrap();

    // Empty path string is treated as no path (None), so should succeed
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["recipeName"], "Test Recipe");
    assert!(json["recipeId"].is_string());

    // Verify file was created at root of recipes directory (not in a category subdirectory)
    let filename = verify_recipe_file_exists_at_root(&temp_dir, "Test Recipe");
    assert!(filename.ends_with(".cook"));

    // Verify file contents
    let contents = read_recipe_file_at_root(&temp_dir, "Test Recipe");
    assert_eq!(contents, content);

    temp_dir
}

#[tokio::test]
async fn test_create_recipe_empty_category_git() {
    let temp_dir = test_create_recipe_empty_category_impl("git").await;

    // Git-specific verification
    let commit_count = count_git_commits(&temp_dir);
    assert!(commit_count > 0, "Expected at least one commit in git repo");
}

#[tokio::test]
async fn test_create_recipe_empty_category_disk() {
    let _temp_dir = test_create_recipe_empty_category_impl("disk").await;
}

async fn test_create_recipe_empty_content_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let payload = serde_json::json!({
        "content": "",
        "path": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_recipe_empty_content_git() {
    test_create_recipe_empty_content_impl("git").await;
}

#[tokio::test]
async fn test_create_recipe_empty_content_disk() {
    test_create_recipe_empty_content_impl("disk").await;
}

// ============================================================================
// RECIPE RETRIEVAL TESTS
// ============================================================================

async fn test_list_recipes_empty_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipes"].as_array().unwrap().len(), 0);
    assert_eq!(json["pagination"]["total"], 0);
}

#[tokio::test]
async fn test_list_recipes_empty_git() {
    test_list_recipes_empty_impl("git").await;
}

#[tokio::test]
async fn test_list_recipes_empty_disk() {
    test_list_recipes_empty_impl("disk").await;
}

async fn test_list_recipes_with_pagination_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("recipe-1", Some("desserts"), "recipe-1.cook"),
            ("recipe-2", Some("desserts"), "recipe-2.cook"),
        ],
    )
    .await;

    // List with default pagination
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipes"].as_array().unwrap().len(), 2);
    assert_eq!(json["pagination"]["total"], 2);
    assert_eq!(json["pagination"]["limit"], 20);
    assert_eq!(json["pagination"]["offset"], 0);
}

#[tokio::test]
async fn test_list_recipes_with_pagination_git() {
    test_list_recipes_with_pagination_impl("git").await;
}

#[tokio::test]
async fn test_list_recipes_with_pagination_disk() {
    test_list_recipes_with_pagination_impl("disk").await;
}

async fn test_list_recipes_with_limit_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("recipe-1", Some("desserts"), "recipe-1.cook"),
            ("recipe-2", Some("desserts"), "recipe-2.cook"),
            ("test-recipe", Some("desserts"), "test-recipe.cook"),
        ],
    )
    .await;

    // List with limit=2
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes?limit=2", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipes"].as_array().unwrap().len(), 2);
    assert_eq!(json["pagination"]["total"], 3);
    assert_eq!(json["pagination"]["limit"], 2);
}

#[tokio::test]
async fn test_list_recipes_with_limit_git() {
    test_list_recipes_with_limit_impl("git").await;
}

#[tokio::test]
async fn test_list_recipes_with_limit_disk() {
    test_list_recipes_with_limit_impl("disk").await;
}

async fn test_get_recipe_not_found_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes/nonexistent", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_recipe_not_found_git() {
    test_get_recipe_not_found_impl("git").await;
}

#[tokio::test]
async fn test_get_recipe_not_found_disk() {
    test_get_recipe_not_found_impl("disk").await;
}

async fn test_get_recipe_by_id_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app1 = build_router();

    // Create a recipe
    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "content": content,
        "path": "desserts"
    });

    let response = app1
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipeId"].as_str().unwrap();

    // Retrieve the recipe
    let app2 = build_router();
    let response = app2
        .oneshot(make_request(
            "GET",
            &format!("/api/v1/recipes/{}", recipe_id),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipeName"], "Test Recipe");
    assert_eq!(json["path"], "desserts");
}

#[tokio::test]
async fn test_get_recipe_by_id_git() {
    test_get_recipe_by_id_impl("git").await;
}

#[tokio::test]
async fn test_get_recipe_by_id_disk() {
    test_get_recipe_by_id_impl("disk").await;
}

// ============================================================================
// RECIPE SEARCH TESTS
// ============================================================================

async fn test_search_recipes_empty_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes/search?q=test", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipes"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_search_recipes_empty_git() {
    test_search_recipes_empty_impl("git").await;
}

#[tokio::test]
async fn test_search_recipes_empty_disk() {
    test_search_recipes_empty_impl("disk").await;
}

async fn test_search_recipes_by_name_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("chocolate-cake", Some("main"), "chocolate-cake.cook"),
            ("vanilla-cake", Some("main"), "vanilla-cake.cook"),
            ("pasta", Some("main"), "pasta.cook"),
        ],
    )
    .await;

    // Search for "cake"
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes/search?q=cake", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let results = json["recipes"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<&str> = results
        .iter()
        .map(|r| r["recipeName"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Chocolate Cake"));
    assert!(names.contains(&"Vanilla Cake"));
}

#[tokio::test]
async fn test_search_recipes_by_name_git() {
    test_search_recipes_by_name_impl("git").await;
}

#[tokio::test]
async fn test_search_recipes_by_name_disk() {
    test_search_recipes_by_name_impl("disk").await;
}

async fn test_search_case_insensitive_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![("chocolate-cake", Some("desserts"), "chocolate-cake.cook")],
    )
    .await;

    // Search with different cases
    let app = build_router();
    let response = app
        .oneshot(make_request(
            "GET",
            "/api/v1/recipes/search?q=CHOCOLATE",
            None,
        ))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipes"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_search_case_insensitive_git() {
    test_search_case_insensitive_impl("git").await;
}

#[tokio::test]
async fn test_search_case_insensitive_disk() {
    test_search_case_insensitive_impl("disk").await;
}

// ============================================================================
// CATEGORY TESTS
// ============================================================================

async fn test_list_categories_empty_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/categories", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["categories"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_categories_empty_git() {
    test_list_categories_empty_impl("git").await;
}

#[tokio::test]
async fn test_list_categories_empty_disk() {
    test_list_categories_empty_impl("disk").await;
}

async fn test_list_categories_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("cake", Some("desserts"), "cake.cook"),
            ("pasta", Some("main"), "pasta.cook"),
            ("test-recipe", Some("appetizers"), "test-recipe.cook"),
        ],
    )
    .await;

    // List categories
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/categories", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let cats = json["categories"].as_array().unwrap();
    assert_eq!(cats.len(), 3);

    let cat_names: Vec<&str> = cats.iter().map(|c| c.as_str().unwrap()).collect();
    assert!(cat_names.contains(&"desserts"));
    assert!(cat_names.contains(&"main"));
    assert!(cat_names.contains(&"appetizers"));
}

#[tokio::test]
async fn test_list_categories_git() {
    test_list_categories_impl("git").await;
}

#[tokio::test]
async fn test_list_categories_disk() {
    test_list_categories_impl("disk").await;
}

async fn test_get_recipes_in_category_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("cake", Some("desserts"), "cake.cook"),
            ("cake", Some("desserts"), "cookie.cook"),
            ("pasta", Some("main"), "pasta.cook"),
            ("pasta", Some("main"), "steak.cook"),
        ],
    )
    .await;

    // Get recipes in desserts category
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/categories/desserts", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let recipes = json["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 2);

    let names: Vec<&str> = recipes
        .iter()
        .map(|r| r["recipeName"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Cake"));
    assert!(names.contains(&"Cake"));
}

#[tokio::test]
async fn test_get_recipes_in_category_git() {
    test_get_recipes_in_category_impl("git").await;
}

#[tokio::test]
async fn test_get_recipes_in_category_disk() {
    test_get_recipes_in_category_impl("disk").await;
}

async fn test_get_category_not_found_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/categories/nonexistent", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_category_not_found_git() {
    test_get_category_not_found_impl("git").await;
}

#[tokio::test]
async fn test_get_category_not_found_disk() {
    test_get_category_not_found_impl("disk").await;
}

// ============================================================================
// RECIPE UPDATE TESTS
// ============================================================================

async fn test_update_recipe_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app1 = build_router();

    // Create a recipe
    let create_content = load_recipe_fixture("original-name");
    let create_payload = serde_json::json!({
        "content": create_content.clone(),
        "path": "desserts"
    });

    let response = app1
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(create_payload),
        ))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipeId"].as_str().unwrap().to_string();

    // Verify original file exists
    verify_recipe_file_exists(&temp_dir, "Original Name", "desserts");
    let original_content = read_recipe_file(&temp_dir, "Original Name", "desserts");
    assert_eq!(original_content, create_content);

    // Update the recipe
    let app2 = build_router();
    let update_content = load_recipe_fixture("updated-name");
    let update_payload = serde_json::json!({
        "content": update_content.clone(),
        "path": "main"
    });

    let response = app2
        .oneshot(make_request(
            "PUT",
            &format!("/api/v1/recipes/{}", recipe_id),
            Some(update_payload.clone()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipeName"], "Updated Name");
    assert_eq!(json["path"], "main");

    // Verify file was updated on disk (moved to new category)
    let filename = verify_recipe_file_exists(&temp_dir, "Updated Name", "main");
    assert!(filename.ends_with(".cook"));

    // Verify file contents were updated
    let updated_file_content = read_recipe_file(&temp_dir, "Updated Name", "main");
    assert_eq!(updated_file_content, update_content);

    // Verify original file is gone from desserts category
    verify_recipe_file_deleted(&temp_dir, "Original Name", "desserts");

    temp_dir
}

#[tokio::test]
async fn test_update_recipe_git() {
    let temp_dir = test_update_recipe_impl("git").await;

    // Git-specific verification
    let commit_count = count_git_commits(&temp_dir);
    assert!(
        commit_count >= 2,
        "Expected at least 2 commits (create + update) in git repo"
    );
}

#[tokio::test]
async fn test_update_recipe_disk() {
    let _temp_dir = test_update_recipe_impl("disk").await;
}

async fn test_update_recipe_not_found_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let payload = serde_json::json!({
        "content": "# Updated\n\n@flour{2%cup}",
        "path": "desserts"
    });

    let response = app
        .oneshot(make_request(
            "PUT",
            "/api/v1/recipes/nonexistent",
            Some(payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_recipe_not_found_git() {
    test_update_recipe_not_found_impl("git").await;
}

#[tokio::test]
async fn test_update_recipe_not_found_disk() {
    test_update_recipe_not_found_impl("disk").await;
}

// ============================================================================
// RECIPE DELETE TESTS
// ============================================================================

async fn test_delete_recipe_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app1 = build_router();

    // Create a recipe
    let content = load_recipe_fixture("to-delete");
    let payload = serde_json::json!({
        "content": content,
        "path": "desserts"
    });

    let response = app1
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipeId"].as_str().unwrap().to_string();

    // Verify file exists before deletion
    verify_recipe_file_exists(&temp_dir, "To Delete", "desserts");

    // Delete the recipe
    let app2 = build_router();
    let response = app2
        .oneshot(make_request(
            "DELETE",
            &format!("/api/v1/recipes/{}", recipe_id),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);

    // Verify file was deleted from disk
    verify_recipe_file_deleted(&temp_dir, "To Delete", "desserts");

    // Verify it's deleted via API
    let app3 = build_router();
    let response = app3
        .oneshot(make_request(
            "GET",
            &format!("/api/v1/recipes/{}", recipe_id),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

    temp_dir
}

#[tokio::test]
async fn test_delete_recipe_git() {
    let temp_dir = test_delete_recipe_impl("git").await;

    // Git-specific verification
    let commit_count = count_git_commits(&temp_dir);
    assert!(
        commit_count >= 2,
        "Expected at least 2 commits (create + delete) in git repo"
    );
}

#[tokio::test]
async fn test_delete_recipe_disk() {
    let _temp_dir = test_delete_recipe_impl("disk").await;
}

async fn test_delete_recipe_not_found_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    let response = app
        .oneshot(make_request("DELETE", "/api/v1/recipes/nonexistent", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_recipe_not_found_git() {
    test_delete_recipe_not_found_impl("git").await;
}

#[tokio::test]
async fn test_delete_recipe_not_found_disk() {
    test_delete_recipe_not_found_impl("disk").await;
}

// ============================================================================
// STATUS AFTER MODIFICATIONS TESTS
// ============================================================================

async fn test_status_updates_with_recipes_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("cake", Some("desserts"), "cake.cook"),
            ("pasta", Some("main"), "pasta.cook"),
        ],
    )
    .await;

    // Check status
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/status", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipe_count"], 2);
    assert_eq!(json["categories"], 2);
}

#[tokio::test]
async fn test_status_updates_with_recipes_git() {
    test_status_updates_with_recipes_impl("git").await;
}

#[tokio::test]
async fn test_status_updates_with_recipes_disk() {
    test_status_updates_with_recipes_impl("disk").await;
}

// ============================================================================
// HIERARCHICAL CATEGORY TESTS
// ============================================================================

async fn test_create_recipe_in_nested_category_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    // Create recipe in nested category
    let content = load_recipe_fixture("chicken-biryani");
    let create_payload = serde_json::json!({
        "content": content,
        "path": "meals/meat/traditional"
    });

    let response = app
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipeName"], "Chicken Biryani");
    assert_eq!(json["path"], "meals/meat/traditional");

    // Verify nested directory structure exists on disk
    verify_recipe_file_exists(&temp_dir, "Chicken Biryani", "meals/meat/traditional");

    temp_dir
}

#[tokio::test]
async fn test_create_recipe_in_nested_category_git() {
    let temp_dir = test_create_recipe_in_nested_category_impl("git").await;

    // Git-specific verification
    let commit_count = count_git_commits(&temp_dir);
    assert!(commit_count > 0);
}

#[tokio::test]
async fn test_create_recipe_in_nested_category_disk() {
    let _temp_dir = test_create_recipe_in_nested_category_impl("disk").await;
}

async fn test_read_recipe_from_nested_category_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![(
            "thai-green-curry",
            Some("meals/asian/thai"),
            "thai-green-curry.cook",
        )],
    )
    .await;

    // Get the recipe by listing all
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipes = json["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 1);

    let recipe_id = recipes[0]["recipeId"].as_str().unwrap().to_string();
    assert_eq!(recipes[0]["recipeName"], "Thai Green Curry");
    assert_eq!(recipes[0]["path"], "meals/asian/thai");

    // Verify content contains expected ingredient
    let app2 = build_router();
    let response = app2
        .oneshot(make_request(
            "GET",
            &format!("/api/v1/recipes/{}", recipe_id),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipeName"], "Thai Green Curry");
    assert_eq!(json["path"], "meals/asian/thai");
    assert!(json["content"]
        .as_str()
        .unwrap()
        .contains("@coconut-milk{400%ml}"));
}

#[tokio::test]
async fn test_read_recipe_from_nested_category_git() {
    test_read_recipe_from_nested_category_impl("git").await;
}

#[tokio::test]
async fn test_read_recipe_from_nested_category_disk() {
    test_read_recipe_from_nested_category_impl("disk").await;
}

async fn test_move_recipe_between_nested_categories_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app1 = build_router();

    // Create recipe in one nested category
    let content = load_recipe_fixture("chocolate-cake");
    let create_payload = serde_json::json!({
        "content": content,
        "path": "desserts/cakes/chocolate"
    });

    let response = app1
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(create_payload),
        ))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipeId"].as_str().unwrap().to_string();

    // Verify it exists in original nested category
    verify_recipe_file_exists(&temp_dir, "Chocolate Cake", "desserts/cakes/chocolate");

    // Move to different nested category
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "path": "desserts/cakes/dark-chocolate"
    });

    let response = app2
        .oneshot(make_request(
            "PUT",
            &format!("/api/v1/recipes/{}", recipe_id),
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["path"], "desserts/cakes/dark-chocolate");

    // Verify file moved to new nested category
    verify_recipe_file_exists(&temp_dir, "Chocolate Cake", "desserts/cakes/dark-chocolate");

    // Verify file no longer exists in original category
    verify_recipe_file_deleted(&temp_dir, "Chocolate Cake", "desserts/cakes/chocolate");

    temp_dir
}

#[tokio::test]
async fn test_move_recipe_between_nested_categories_git() {
    let _temp_dir = test_move_recipe_between_nested_categories_impl("git").await;
}

#[tokio::test]
async fn test_move_recipe_between_nested_categories_disk() {
    let _temp_dir = test_move_recipe_between_nested_categories_impl("disk").await;
}

async fn test_get_recipes_from_nested_category_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("pad-thai", Some("meals/asian/thai"), "pad-thai.cook"),
            ("green-curry", Some("meals/asian/thai"), "green-curry.cook"),
            (
                "spaghetti",
                Some("meals/european/italian"),
                "spaghetti.cook",
            ),
        ],
    )
    .await;

    // Get recipes from nested Thai category (URL-encoded)
    let app = build_router();
    let response = app
        .oneshot(make_request(
            "GET",
            "/api/v1/categories/meals%2Fasian%2Fthai",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["path"], "meals/asian/thai");
    assert_eq!(json["count"], 2);

    let recipes = json["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 2);

    let names: Vec<String> = recipes
        .iter()
        .map(|r| r["recipeName"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"Pad Thai".to_string()));
    assert!(names.contains(&"Green Curry".to_string()));
}

#[tokio::test]
async fn test_get_recipes_from_nested_category_git() {
    test_get_recipes_from_nested_category_impl("git").await;
}

#[tokio::test]
async fn test_get_recipes_from_nested_category_disk() {
    test_get_recipes_from_nested_category_impl("disk").await;
}

async fn test_move_recipe_between_flat_and_nested_category_impl(backend: &str) -> TempDir {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app1 = build_router();

    // Create recipe in flat category
    let content = load_recipe_fixture("vanilla-cake");
    let create_payload = serde_json::json!({
        "content": content,
        "path": "desserts"
    });

    let response = app1
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(create_payload),
        ))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipeId"].as_str().unwrap().to_string();

    // Verify in flat category
    verify_recipe_file_exists(&temp_dir, "Vanilla Cake", "desserts");

    // Move to nested category
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "path": "desserts/cakes/vanilla"
    });

    let response = app2
        .oneshot(make_request(
            "PUT",
            &format!("/api/v1/recipes/{}", recipe_id),
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["path"], "desserts/cakes/vanilla");

    // Verify moved to nested category
    verify_recipe_file_exists(&temp_dir, "Vanilla Cake", "desserts/cakes/vanilla");
    verify_recipe_file_deleted(&temp_dir, "Vanilla Cake", "desserts");

    temp_dir
}

#[tokio::test]
async fn test_move_recipe_between_flat_and_nested_category_git() {
    let _temp_dir = test_move_recipe_between_flat_and_nested_category_impl("git").await;
}

#[tokio::test]
async fn test_move_recipe_between_flat_and_nested_category_disk() {
    let _temp_dir = test_move_recipe_between_flat_and_nested_category_impl("disk").await;
}

async fn test_list_categories_includes_nested_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_seeded_fixtures(
        backend,
        vec![
            ("tiramisu", Some("desserts/cakes/italian"), "tiramisu.cook"),
            (
                "cheesecake",
                Some("desserts/cakes/american"),
                "cheesecake.cook",
            ),
            ("flan", Some("desserts/custards"), "flan.cook"),
        ],
    )
    .await;

    // List all categories
    let app = build_router();
    let response = app
        .oneshot(make_request("GET", "/api/v1/categories", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let categories: Vec<String> = json["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();

    // Should have nested categories, not flattened
    assert!(categories.contains(&"desserts/cakes/italian".to_string()));
    assert!(categories.contains(&"desserts/cakes/american".to_string()));
    assert!(categories.contains(&"desserts/custards".to_string()));
}

#[tokio::test]
async fn test_list_categories_includes_nested_git() {
    test_list_categories_includes_nested_impl("git").await;
}

#[tokio::test]
async fn test_list_categories_includes_nested_disk() {
    test_list_categories_includes_nested_impl("disk").await;
}

async fn test_move_between_different_category_structures_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app1 = build_router();

    // Create recipe in one category structure
    let content = load_recipe_fixture("authors-dinner");
    let create_payload = serde_json::json!({
        "content": content,
        "path": "author1/dinner/meat"
    });

    let response = app1
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipeId"].as_str().unwrap().to_string();

    assert_eq!(json["path"], "author1/dinner/meat");

    // Move to completely different category structure
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "path": "author2/meat/dinner"
    });

    let response = app2
        .oneshot(make_request(
            "PUT",
            &format!("/api/v1/recipes/{}", recipe_id),
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    // Verify category updated to new structure
    assert_eq!(json["path"], "author2/meat/dinner");
    assert_eq!(json["recipeName"], "Author's Dinner");
}

#[tokio::test]
async fn test_move_between_different_category_structures_git() {
    test_move_between_different_category_structures_impl("git").await;
}

#[tokio::test]
async fn test_move_between_different_category_structures_disk() {
    test_move_between_different_category_structures_impl("disk").await;
}

// ============================================================================
// YAML FRONT MATTER & FILE RENAMING TESTS (PHASE 2.4)
// ============================================================================

async fn test_create_recipe_missing_yaml_front_matter_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    // Content without YAML front matter (no title field)
    let content = "---\ndescription: This is just a description\n---\n\nNo title provided.";
    let payload = serde_json::json!({
        "content": content,
        "path": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    // Should return 400 Bad Request (invalid recipe content)
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

    let body = extract_response_body(response).await;
    // Error message should mention title or front matter
    assert!(
        body.to_lowercase().contains("title") || body.to_lowercase().contains("front matter"),
        "Error body: {}",
        body
    );
}

#[tokio::test]
async fn test_create_recipe_missing_yaml_front_matter_git() {
    test_create_recipe_missing_yaml_front_matter_impl("git").await;
}

#[tokio::test]
async fn test_create_recipe_missing_yaml_front_matter_disk() {
    test_create_recipe_missing_yaml_front_matter_impl("disk").await;
}

async fn test_create_recipe_with_valid_yaml_front_matter_impl(backend: &str) {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    // Content with valid YAML front matter including title
    let content = "---\ntitle: Chocolate Cake\ndescription: Rich chocolate cake\n---\n\nMix flour with cocoa.";
    let payload = serde_json::json!({
        "content": content,
        "path": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    // The name in the response should be from YAML title, not the request name
    assert_eq!(json["recipeName"], "Chocolate Cake");
    assert_eq!(json["path"], "desserts");
    assert!(json["recipeId"].is_string());

    // Verify file was created with name derived from title
    let filename = verify_recipe_file_exists(&temp_dir, "Chocolate Cake", "desserts");
    assert_eq!(filename, "chocolate-cake.cook");

    let file_contents = read_recipe_file(&temp_dir, "Chocolate Cake", "desserts");
    assert_eq!(file_contents, content);
}

#[tokio::test]
async fn test_create_recipe_with_valid_yaml_front_matter_git() {
    test_create_recipe_with_valid_yaml_front_matter_impl("git").await;
}

#[tokio::test]
async fn test_create_recipe_with_valid_yaml_front_matter_disk() {
    test_create_recipe_with_valid_yaml_front_matter_impl("disk").await;
}

async fn test_update_recipe_title_causes_filename_change_impl(backend: &str) {
    let (build_router, temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    // Step 1: Create initial recipe
    let initial_content = "---\ntitle: Brownie\n---\n\nChocolate brownie recipe.";
    let payload = serde_json::json!({
        "content": initial_content,
        "path": "desserts"
    });

    let response = app
        .clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let recipe_id = json["recipeId"].as_str().unwrap();
    let initial_filename = "brownie.cook";

    // Verify initial file exists
    verify_recipe_file_exists(&temp_dir, "Brownie", "desserts");

    // Step 2: Update recipe with new title
    let updated_content = "---\ntitle: Fudgy Brownie\n---\n\nExtra fudgy chocolate brownie recipe.";
    let update_payload = serde_json::json!({
        "content": updated_content
    });

    let response = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/v1/recipes/{}", recipe_id),
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    // Name should be updated to new title
    assert_eq!(json["recipeName"], "Fudgy Brownie");
    let new_recipe_id = json["recipeId"].as_str().unwrap();

    // recipe_id should change (because git_path changed)
    assert_ne!(recipe_id, new_recipe_id);

    // Step 3: Verify file was renamed on disk
    let new_filename = verify_recipe_file_exists(&temp_dir, "Fudgy Brownie", "desserts");
    assert_eq!(new_filename, "fudgy-brownie.cook");
    assert_ne!(initial_filename, new_filename);

    // Verify old file no longer exists
    let files = std::fs::read_dir(temp_dir.path().join("recipes/desserts"))
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                path.file_name().map(|n| n.to_str().unwrap().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert!(!files.contains(&initial_filename.to_string()));
    assert!(files.contains(&new_filename.to_string()));
}

#[tokio::test]
async fn test_update_recipe_title_causes_filename_change_git() {
    test_update_recipe_title_causes_filename_change_impl("git").await;
}

#[tokio::test]
async fn test_update_recipe_title_causes_filename_change_disk() {
    test_update_recipe_title_causes_filename_change_impl("disk").await;
}

async fn test_id_change_on_rename_scenario_impl(backend: &str) {
    let (build_router, _temp_dir) = setup_api_with_storage(backend).await;
    let app = build_router();

    // Step 1: Create recipe with initial title
    let initial_content = "---\ntitle: Chocolate Cake\n---\n\nSimple chocolate cake.";
    let payload = serde_json::json!({
        "content": initial_content,
        "path": "desserts"
    });

    let response = app
        .clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let initial_recipe_id = json["recipeId"].as_str().unwrap().to_string();
    println!("Initial recipe ID: {}", initial_recipe_id);

    // Step 2: Update recipe content with new title
    let new_content = "---\ntitle: Dark Chocolate Cake\n---\n\nRich dark chocolate cake.";
    let update_payload = serde_json::json!({
        "content": new_content
    });

    let response = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/v1/recipes/{}", initial_recipe_id),
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let new_recipe_id = json["recipeId"].as_str().unwrap().to_string();
    println!("New recipe ID: {}", new_recipe_id);

    // Verify recipe_id changed
    assert_ne!(initial_recipe_id, new_recipe_id);
    assert_eq!(json["recipeName"], "Dark Chocolate Cake");

    // Step 3: Try to access with old recipe_id - should return 404
    let response = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/v1/recipes/{}", initial_recipe_id),
            None,
        ))
        .await
        .unwrap();

    // Old ID should not be valid anymore
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

    // Step 4: Verify new recipe_id works
    let response = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/v1/recipes/{}", new_recipe_id),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["recipeName"], "Dark Chocolate Cake");
}

#[tokio::test]
async fn test_id_change_on_rename_scenario_git() {
    test_id_change_on_rename_scenario_impl("git").await;
}

#[tokio::test]
async fn test_id_change_on_rename_scenario_disk() {
    test_id_change_on_rename_scenario_impl("disk").await;
}
