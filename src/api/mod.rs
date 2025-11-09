pub mod handlers;
pub mod models;
pub mod responses;

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::repository::RecipeRepository;

/// Build the API router with all routes
pub fn build_router(repo: Arc<RecipeRepository>) -> Router {
    // Split routes: those that don't need state and those that do
    let public_routes = Router::new().route("/health", get(handlers::health_check));

    let api_routes = Router::new()
        .route("/status", get(handlers::status))
        // Recipe CRUD endpoints
        .route("/recipes", post(handlers::create_recipe))
        .route("/recipes", get(handlers::list_recipes))
        .route("/recipes/search", get(handlers::search_recipes))
        .route("/recipes/:recipe_id", get(handlers::get_recipe))
        .route("/recipes/:recipe_id", put(handlers::update_recipe))
        .route("/recipes/:recipe_id", delete(handlers::delete_recipe))
        // Category endpoints
        .route("/categories", get(handlers::list_categories))
        .route("/categories/:name", get(handlers::get_category_recipes))
        .with_state(repo);

    // Combine routers
    Router::new()
        .merge(public_routes)
        .nest("/api/v1", api_routes)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit for recipe content
        .layer(CorsLayer::permissive())
}
