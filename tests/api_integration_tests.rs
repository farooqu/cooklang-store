use cooklang_backend::{api, repository::RecipeRepository};
use serde_json::Value;
use std::sync::Arc;
use std::fs;
use tempfile::TempDir;
use tower::util::ServiceExt;

async fn setup_api() -> (impl Fn() -> axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo = RecipeRepository::new(temp_dir.path())
        .await
        .expect("Failed to create repo");

    let repo_arc = Arc::new(repo);

    let build_router = move || api::build_router(repo_arc.clone());

    (build_router, temp_dir)
}

fn make_request(
    method: &str,
    uri: &str,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder().method(method).uri(uri);

    let request = if let Some(json_body) = body {
        builder = builder.header("content-type", "application/json");
        builder
            .body(axum::body::Body::from(json_body.to_string()))
            .unwrap()
    } else {
        builder
            .body(axum::body::Body::empty())
            .unwrap()
    };

    request
}

async fn extract_response_body(response: axum::http::Response<axum::body::Body>) -> String {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body_bytes.to_vec()).unwrap()
}

// ============================================================================
// GIT REPOSITORY VERIFICATION HELPERS
// ============================================================================

fn verify_recipe_file_exists(temp_dir: &TempDir, recipe_name: &str, category: &str) -> String {
    let recipes_dir = temp_dir.path().join("recipes").join(category);
    assert!(
        recipes_dir.exists(),
        "Category directory doesn't exist: {}",
        recipes_dir.display()
    );

    // Find the recipe file (could be recipe-name.cook or recipe-name-2.cook if duplicate)
    let name_slug = recipe_name.to_lowercase().replace(" ", "-");
    let files: Vec<_> = fs::read_dir(&recipes_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "cook").unwrap_or(false) {
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                if filename.starts_with(&name_slug) {
                    Some(filename)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    assert!(
        !files.is_empty(),
        "Recipe file not found for '{}' in category '{}'",
        recipe_name,
        category
    );

    files[0].clone()
}

fn read_recipe_file(temp_dir: &TempDir, recipe_name: &str, category: &str) -> String {
    let filename = verify_recipe_file_exists(temp_dir, recipe_name, category);
    let path = temp_dir.path().join("recipes").join(category).join(&filename);
    fs::read_to_string(&path).expect("Failed to read recipe file")
}

fn count_git_commits(temp_dir: &TempDir) -> usize {
    let repo = git2::Repository::open(temp_dir.path()).expect("Failed to open git repo");
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk.count()
}

fn verify_recipe_file_deleted(temp_dir: &TempDir, recipe_name: &str, category: &str) {
    let recipes_dir = temp_dir.path().join("recipes").join(category);
    if !recipes_dir.exists() {
        return; // Category directory removed entirely
    }

    let name_slug = recipe_name.to_lowercase().replace(" ", "-");
    let files: Vec<_> = fs::read_dir(&recipes_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "cook").unwrap_or(false) {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if filename.starts_with(&name_slug) {
                    Some(filename.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    assert!(
        files.is_empty(),
        "Recipe file still exists after deletion: {:?}",
        files
    );
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

    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": "# Test Recipe\n\n@ingredient{} flour",
        "category": "desserts"
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload.clone())))
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
    assert_eq!(contents, payload["content"].as_str().unwrap());
}

#[tokio::test]
async fn test_create_recipe_with_comment() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let payload = serde_json::json!({
        "name": "Chocolate Cake",
        "content": "# Chocolate Cake\n\n@flour{2%cup}",
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

    let payload = serde_json::json!({
        "name": "",
        "content": "# Test\n\n@ingredient{} flour",
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
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": "# Test\n\n@ingredient{} flour",
        "category": ""
    });

    let response = app
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    // Empty category string is treated as no category (None), so should succeed
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
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
    let payload1 = serde_json::json!({
        "name": "Recipe 1",
        "content": "# Recipe 1\n\n@flour{1%cup}",
        "category": "desserts"
    });
    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload1)))
        .await
        .unwrap();

    // Create second recipe
    let app2 = build_router();
    let payload2 = serde_json::json!({
        "name": "Recipe 2",
        "content": "# Recipe 2\n\n@flour{2%cup}",
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
    for i in 1..=3 {
        let payload = serde_json::json!({
            "name": format!("Recipe {}", i),
            "content": format!("# Recipe {}\n\n@flour{{{}%cup}}", i, i),
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
    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": "# Test Recipe\n\n@flour{2%cup}",
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
        ("Chocolate Cake", "# Chocolate Cake\n\n@flour{2%cup}"),
        ("Vanilla Cake", "# Vanilla Cake\n\n@flour{2%cup}"),
        ("Pasta Carbonara", "# Pasta\n\n@pasta{400%g}"),
    ];

    for (name, content) in recipes {
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

    let payload = serde_json::json!({
        "name": "Chocolate Cake",
        "content": "# Chocolate Cake\n\n@flour{2%cup}",
        "category": "desserts"
    });

    app1.oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
        .await
        .unwrap();

    // Search with different cases
    let app2 = build_router();
    let response = app2
        .oneshot(make_request("GET", "/api/v1/recipes/search?q=CHOCOLATE", None))
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
    let categories = vec!["desserts", "main", "appetizers"];
    for (i, category) in categories.iter().enumerate() {
        let payload = serde_json::json!({
            "name": format!("Recipe {}", i),
            "content": format!("# Recipe {}\n\n@flour{{{}%cup}}", i, i),
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
    let dessert_recipes = vec!["Cake", "Cookie"];
    let main_recipes = vec!["Pasta", "Steak"];

    for name in &dessert_recipes {
        let payload = serde_json::json!({
            "name": name,
            "content": format!("# {}\n\n@flour{{1%cup}}", name),
            "category": "desserts"
        });
        app1.clone()
            .oneshot(make_request("POST", "/api/v1/recipes", Some(payload)))
            .await
            .unwrap();
    }

    for name in &main_recipes {
        let payload = serde_json::json!({
            "name": name,
            "content": format!("# {}\n\n@meat{{500%g}}", name),
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
    let create_payload = serde_json::json!({
        "name": "Original Name",
        "content": "# Original\n\n@flour{1%cup}",
        "category": "desserts"
    });

    let response = app1
        .oneshot(make_request("POST", "/api/v1/recipes", Some(create_payload)))
        .await
        .unwrap();

    let body = extract_response_body(response).await;
    let json: Value = serde_json::from_str(&body).unwrap();
    let recipe_id = json["recipe_id"].as_str().unwrap().to_string();

    // Verify original file exists
    verify_recipe_file_exists(&temp_dir, "Original Name", "desserts");
    let original_content = read_recipe_file(&temp_dir, "Original Name", "desserts");
    assert_eq!(original_content, "# Original\n\n@flour{1%cup}");

    // Update the recipe
    let app2 = build_router();
    let update_payload = serde_json::json!({
        "name": "Updated Name",
        "content": "# Updated\n\n@flour{2%cup}",
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
    let updated_content = read_recipe_file(&temp_dir, "Updated Name", "main");
    assert_eq!(updated_content, update_payload["content"].as_str().unwrap());

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
    let payload = serde_json::json!({
        "name": "To Delete",
        "content": "# Delete me\n\n@flour{1%cup}",
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
    let payload1 = serde_json::json!({
        "name": "Cake",
        "content": "# Cake\n\n@flour{2%cup}",
        "category": "desserts"
    });
    app1.clone()
        .oneshot(make_request("POST", "/api/v1/recipes", Some(payload1)))
        .await
        .unwrap();

    let payload2 = serde_json::json!({
        "name": "Pasta",
        "content": "# Pasta\n\n@pasta{400%g}",
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
