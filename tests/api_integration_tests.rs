use cooklang_backend::{api, repository::RecipeRepository};
use std::sync::Arc;
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

#[tokio::test]
async fn test_health_check() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_status_endpoint() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/status")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_create_recipe() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let payload = serde_json::json!({
        "name": "Test Recipe",
        "content": "# Test Recipe\n\n@ingredient{} flour",
        "category": "desserts"
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/recipes")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_recipe_validation() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    // Empty name
    let payload = serde_json::json!({
        "name": "",
        "content": "# Test\n\n@ingredient{} flour",
        "category": "desserts"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/recipes")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_categories_empty() {
    let (build_router, _temp_dir) = setup_api().await;
    let app = build_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/categories")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
