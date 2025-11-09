mod common;

use common::*;
use serde_json::Value;
use tempfile::TempDir;
use tower::util::ServiceExt;

async fn setup_api() -> (impl Fn() -> axum::Router, TempDir) {
    setup_api_with_storage("git").await
}

// ============================================================================
// HEALTH & STATUS TESTS
// ============================================================================

#[tokio::test]
async fn test_health_check() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/health", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_status_endpoint() {
    let (build_router, _temp_dir) = setup_api().await;
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

// ============================================================================
// RECIPE CREATION TESTS
// ============================================================================

#[tokio::test]
async fn test_create_recipe() {
    let (build_router, temp_dir) = setup_api().await;
    let app = build_router();

    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": content.clone(),
        "category": "desserts"
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

    assert_eq!(json["name"], "Test Recipe");
    assert_eq!(json["category"], "desserts");
    assert!(json["recipe_id"].is_string());

    // Verify file was created in git repo
    let filename = verify_recipe_file_exists(&temp_dir, "Test Recipe", "desserts");
    assert!(filename.ends_with(".cook"));

    // Verify file contents
    let contents = read_recipe_file(&temp_dir, "Test Recipe", "desserts");
    assert_eq!(contents, content);
}

#[tokio::test]
async fn test_create_recipe_with_comment() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let content = load_recipe_fixture("chocolate-cake");
    let payload = serde_json::json!({
        "name": "Chocolate Cake",
        "content": content,
        "category": "desserts",
        "comment": "Classic chocolate recipe"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["name"], "Chocolate Cake");
    assert_eq!(json["category"], "desserts");
}

#[tokio::test]
async fn test_create_recipe_empty_name() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "name": "",
        "content": content,
        "category": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_recipe_empty_category() {
    let (build_router, temp_dir) = setup_api().await;
    let app = build_router();

    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": content.clone(),
        "category": ""
    });

    let response = app
        .oneshot(make_request(
            "POST",
            "/api/v1/recipes",
            Some(payload.clone()),
        ))
        .await
        .unwrap();

    // Empty category string is treated as no category (None), so should succeed
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["name"], "Test Recipe");
    assert!(json["recipe_id"].is_string());

    // Verify file was created at root of recipes directory (not in a category subdirectory)
    let filename = verify_recipe_file_exists_at_root(&temp_dir, "Test Recipe");
    assert!(filename.ends_with(".cook"));

    // Verify file contents
    let contents = read_recipe_file_at_root(&temp_dir, "Test Recipe");
    assert_eq!(contents, content);
}

#[tokio::test]
async fn test_create_recipe_empty_content() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": "",
        "category": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

// ============================================================================
// RECIPE RETRIEVAL TESTS
// ============================================================================

#[tokio::test]
async fn test_list_recipes_empty() {
    let (build_router, _temp_dir) = setup_api().await;
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
async fn test_list_recipes_with_pagination() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create first recipe
    let content1 = load_recipe_fixture("recipe-1");
    let payload1 = serde_json::json!({
        "name": "Recipe 1",
        "content": content1,
        "category": "desserts"
    });
    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload1)))
        .await
        .unwrap();

    // Create second recipe
    let app2 = build_router();
    let content2 = load_recipe_fixture("recipe-2");
    let payload2 = serde_json::json!({
        "name": "Recipe 2",
        "content": content2,
        "category": "desserts"
    });
    app2.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload2)))
        .await
        .unwrap();

    // List with default pagination
    let app3 = build_router();
    let response = app3
        .clone()
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
async fn test_list_recipes_with_limit() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create 3 recipes
    let fixture_names = vec!["recipe-1", "recipe-2", "test-recipe"];
    for (i, fixture_name) in fixture_names.iter().enumerate() {
        let content = load_recipe_fixture(fixture_name);
        let payload = serde_json::json!({
            "name": format!("Recipe {}", i + 1),
            "content": content,
            "category": "desserts"
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    // List with limit=2
    let app2 = build_router();
    let response = app2
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
async fn test_get_recipe_not_found() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/recipes/nonexistent", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_recipe_by_id() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create a recipe
    let content = load_recipe_fixture("test-recipe");
    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": content,
        "category": "desserts"
    });

    let response = app1
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipe_id"].as_str().unwrap();

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

    assert_eq!(json["name"], "Test Recipe");
    assert_eq!(json["category"], "desserts");
}

// ============================================================================
// RECIPE SEARCH TESTS
// ============================================================================

#[tokio::test]
async fn test_search_recipes_empty() {
    let (build_router, _temp_dir) = setup_api().await;
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
async fn test_search_recipes_by_name() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipes with different names
    let recipes = vec![
        ("Chocolate Cake", "chocolate-cake"),
        ("Vanilla Cake", "vanilla-cake"),
        ("Pasta Carbonara", "pasta"),
    ];

    for (name, fixture_name) in recipes {
        let content = load_recipe_fixture(fixture_name);
        let payload = serde_json::json!({
            "name": name,
            "content": content,
            "category": "main"
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    // Search for "cake"
    let app2 = build_router();
    let response = app2
        .oneshot(make_request("GET", "/api/v1/recipes/search?q=cake", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    let results = json["recipes"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<&str> = results
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Chocolate Cake"));
    assert!(names.contains(&"Vanilla Cake"));
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    let content = load_recipe_fixture("chocolate-cake");
    let payload = serde_json::json!({
        "name": "Chocolate Cake",
        "content": content,
        "category": "desserts"
    });

    app1.oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    // Search with different cases
    let app2 = build_router();
    let response = app2
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

// ============================================================================
// CATEGORY TESTS
// ============================================================================

#[tokio::test]
async fn test_list_categories_empty() {
    let (build_router, _temp_dir) = setup_api().await;
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
async fn test_list_categories() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipes in different categories
    let recipes = vec![
        ("Cake", "cake", "desserts"),
        ("Pasta", "pasta", "main"),
        ("Test Recipe", "test-recipe", "appetizers"),
    ];
    for (name, fixture_name, category) in recipes {
        let content = load_recipe_fixture(fixture_name);
        let payload = serde_json::json!({
            "name": name,
            "content": content,
            "category": category
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    // List categories
    let app2 = build_router();
    let response = app2
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
async fn test_get_recipes_in_category() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipes in different categories
    let dessert_recipes = vec![("Cake", "cake"), ("Cookie", "cake")];
    let main_recipes = vec![("Pasta", "pasta"), ("Steak", "pasta")];

    for (name, fixture_name) in &dessert_recipes {
        let content = load_recipe_fixture(fixture_name);
        let payload = serde_json::json!({
            "name": name,
            "content": content,
            "category": "desserts"
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    for (name, fixture_name) in &main_recipes {
        let content = load_recipe_fixture(fixture_name);
        let payload = serde_json::json!({
            "name": name,
            "content": content,
            "category": "main"
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    // Get recipes in desserts category
    let app2 = build_router();
    let response = app2
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
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Cake"));
    assert!(names.contains(&"Cookie"));
}

#[tokio::test]
async fn test_get_category_not_found() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(make_request("GET", "/api/v1/categories/nonexistent", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

// ============================================================================
// RECIPE UPDATE TESTS
// ============================================================================

#[tokio::test]
async fn test_update_recipe() {
    let (build_router, temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create a recipe
    let create_content = load_recipe_fixture("original-name");
    let create_payload = serde_json::json!({
        "name": "Original Name",
        "content": create_content.clone(),
        "category": "desserts"
    });

    let response = app1
    .oneshot(make_request(
    "POST",
    "/api/v1/recipes",
    Some(create_payload),
    ))
    .await
    .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED, "Failed to create recipe: response status = {}", response.status());

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

    // Verify original file exists
    verify_recipe_file_exists(&temp_dir, "Original Name", "desserts");
    let original_content = read_recipe_file(&temp_dir, "Original Name", "desserts");
    assert_eq!(original_content, create_content);

    // Update the recipe
    let app2 = build_router();
    let update_content = load_recipe_fixture("updated-name");
    let update_payload = serde_json::json!({
        "name": "Updated Name",
        "content": update_content.clone(),
        "category": "main"
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

    assert_eq!(json["name"], "Updated Name");
    assert_eq!(json["category"], "main");

    // Verify file was updated in git repo (moved to new category)
    let filename = verify_recipe_file_exists(&temp_dir, "Updated Name", "main");
    assert!(filename.ends_with(".cook"));

    // Verify file contents were updated
    let updated_file_content = read_recipe_file(&temp_dir, "Updated Name", "main");
    assert_eq!(updated_file_content, update_content);

    // Verify original file is gone from desserts category
    verify_recipe_file_deleted(&temp_dir, "Original Name", "desserts");
}

#[tokio::test]
async fn test_update_recipe_not_found() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let payload = serde_json::json!({
        "name": "Updated",
        "content": "# Updated\n\n@flour{2%cup}",
        "category": "desserts"
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

// ============================================================================
// RECIPE DELETE TESTS
// ============================================================================

#[tokio::test]
async fn test_delete_recipe() {
    let (build_router, temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create a recipe
    let content = load_recipe_fixture("to-delete");
    let payload = serde_json::json!({
        "name": "To Delete",
        "content": content,
        "category": "desserts"
    });

    let response = app1
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

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

    // Verify file was deleted from git repo
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
}

#[tokio::test]
async fn test_delete_recipe_not_found() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(make_request("DELETE", "/api/v1/recipes/nonexistent", None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

// ============================================================================
// STATUS AFTER MODIFICATIONS TESTS
// ============================================================================

#[tokio::test]
async fn test_status_updates_with_recipes() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create 2 recipes in different categories
    let content1 = load_recipe_fixture("cake");
    let payload1 = serde_json::json!({
        "name": "Cake",
        "content": content1,
        "category": "desserts"
    });
    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload1)))
        .await
        .unwrap();

    let content2 = load_recipe_fixture("pasta");
    let payload2 = serde_json::json!({
        "name": "Pasta",
        "content": content2,
        "category": "main"
    });
    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload2)))
        .await
        .unwrap();

    // Check status
    let app2 = build_router();
    let response = app2
        .oneshot(make_request("GET", "/api/v1/status", None))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["recipe_count"], 2);
    assert_eq!(json["categories"], 2);
}

// ============================================================================
// HIERARCHICAL CATEGORY TESTS
// ============================================================================

#[tokio::test]
async fn test_create_recipe_in_nested_category() {
    let (build_router, temp_dir) = setup_api().await;
    let app = build_router();

    // Create recipe in nested category
    let content = load_recipe_fixture("chicken-biryani");
    let create_payload = serde_json::json!({
        "name": "Chicken Biryani",
        "content": content,
        "category": "meals/meat/traditional"
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

    assert_eq!(json["name"], "Chicken Biryani");
    assert_eq!(json["category"], "meals/meat/traditional");

    // Verify nested directory structure exists on disk
    verify_recipe_file_exists(&temp_dir, "Chicken Biryani", "meals/meat/traditional");
}

#[tokio::test]
async fn test_read_recipe_from_nested_category() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipe in nested category
    let content = load_recipe_fixture("thai-green-curry");
    let create_payload = serde_json::json!({
        "name": "Thai Green Curry",
        "content": content,
        "category": "meals/asian/thai"
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
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

    // Read the recipe back
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

    assert_eq!(json["name"], "Thai Green Curry");
    assert_eq!(json["category"], "meals/asian/thai");
    assert!(json["content"]
    .as_str()
    .unwrap()
    .contains("@coconut-milk{400%ml}"));
}

#[tokio::test]
async fn test_move_recipe_between_nested_categories() {
    let (build_router, temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipe in one nested category
    let content = load_recipe_fixture("chocolate-cake");
    let create_payload = serde_json::json!({
        "name": "Chocolate Cake",
        "content": content,
        "category": "desserts/cakes/chocolate"
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
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

    // Verify it exists in original nested category
    verify_recipe_file_exists(&temp_dir, "Chocolate Cake", "desserts/cakes/chocolate");

    // Move to different nested category
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "category": "desserts/cakes/dark-chocolate"
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
    assert_eq!(json["category"], "desserts/cakes/dark-chocolate");

    // Verify file moved to new nested category
    verify_recipe_file_exists(&temp_dir, "Chocolate Cake", "desserts/cakes/dark-chocolate");

    // Verify file no longer exists in original category
    verify_recipe_file_deleted(&temp_dir, "Chocolate Cake", "desserts/cakes/chocolate");
}

#[tokio::test]
async fn test_get_recipes_from_nested_category() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create multiple recipes in nested category
    let content1 = load_recipe_fixture("pad-thai");
    let payload1 = serde_json::json!({
        "name": "Pad Thai",
        "content": content1,
        "category": "meals/asian/thai"
    });

    let content2 = load_recipe_fixture("green-curry");
    let payload2 = serde_json::json!({
        "name": "Green Curry",
        "content": content2,
        "category": "meals/asian/thai"
    });

    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload1)))
        .await
        .unwrap();

    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload2)))
        .await
        .unwrap();

    // Create recipe in different category to ensure filtering works
    let content3 = load_recipe_fixture("spaghetti");
    let payload3 = serde_json::json!({
        "name": "Spaghetti",
        "content": content3,
        "category": "meals/european/italian"
    });

    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload3)))
        .await
        .unwrap();

    // Get recipes from nested Thai category (URL-encoded)
    let app2 = build_router();
    let response = app2
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

    assert_eq!(json["category"], "meals/asian/thai");
    assert_eq!(json["count"], 2);

    let recipes = json["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 2);

    let names: Vec<String> = recipes
        .iter()
        .map(|r| r["name"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"Pad Thai".to_string()));
    assert!(names.contains(&"Green Curry".to_string()));
}

#[tokio::test]
async fn test_move_recipe_between_flat_and_nested_category() {
    let (build_router, temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipe in flat category
    let content = load_recipe_fixture("vanilla-cake");
    let create_payload = serde_json::json!({
        "name": "Vanilla Cake",
        "content": content,
        "category": "desserts"
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
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

    // Verify in flat category
    verify_recipe_file_exists(&temp_dir, "Vanilla Cake", "desserts");

    // Move to nested category
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "category": "desserts/cakes/vanilla"
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
    assert_eq!(json["category"], "desserts/cakes/vanilla");

    // Verify moved to nested category
    verify_recipe_file_exists(&temp_dir, "Vanilla Cake", "desserts/cakes/vanilla");
    verify_recipe_file_deleted(&temp_dir, "Vanilla Cake", "desserts");
}

#[tokio::test]
async fn test_list_categories_includes_nested() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipes in various nested categories
    let recipes = vec![
        ("Tiramisu", "tiramisu", "desserts/cakes/italian"),
        ("Cheesecake", "cheesecake", "desserts/cakes/american"),
        ("Flan", "flan", "desserts/custards"),
    ];

    for (name, fixture_name, category) in recipes {
        let content = load_recipe_fixture(fixture_name);
        let payload = serde_json::json!({
            "name": name,
            "content": content,
            "category": category
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    // List all categories
    let app2 = build_router();
    let response = app2
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
async fn test_move_between_different_category_structures() {
    let (build_router, _temp_dir) = setup_api().await;
    let app1 = build_router();

    // Create recipe in one category structure
    let content = load_recipe_fixture("authors-dinner");
    let create_payload = serde_json::json!({
        "name": "Author's Dinner",
        "content": content,
        "category": "author1/dinner/meat"
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
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

    assert_eq!(json["category"], "author1/dinner/meat");

    // Move to completely different category structure
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "category": "author2/meat/dinner"
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
    assert_eq!(json["category"], "author2/meat/dinner");
    assert_eq!(json["name"], "Author's Dinner");
}
