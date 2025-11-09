use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    cache::generate_recipe_id, parser::extract_recipe_title, repository::RecipeRepository,
};

use super::{
    models::{CreateRecipeRequest, ListQuery, PaginationInfo, SearchQuery, UpdateRecipeRequest},
    responses::*,
};

/// Health check endpoint - returns simple OK response
pub async fn health_check() -> &'static str {
    "OK"
}

/// Status endpoint - returns server status and recipe count
pub async fn status(State(repo): State<Arc<RecipeRepository>>) -> Json<StatusResponse> {
    let recipes = repo.list_all();
    let categories = repo.get_categories();

    Json(StatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        recipe_count: recipes.len(),
        categories: categories.len(),
    })
}

/// Create a new recipe
pub async fn create_recipe(
    State(repo): State<Arc<RecipeRepository>>,
    Json(payload): Json<CreateRecipeRequest>,
) -> Result<(StatusCode, Json<RecipeResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate content is not empty
    if payload.content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "Recipe content cannot be empty",
            )),
        ));
    }

    // Extract title from content (validates YAML front matter exists)
    let recipe_title = match extract_recipe_title(&payload.content) {
        Ok(title) => title,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "validation_error",
                    format!(
                        "Recipe content must include YAML front matter with 'title' field: {}",
                        e
                    ),
                )),
            ));
        }
    };

    // Default path to empty string (root) if not provided
    let path = payload
        .path
        .as_deref()
        .and_then(|p| if p.trim().is_empty() { None } else { Some(p) });

    // Create recipe
    match repo
        .create_with_author_and_comment(
            &recipe_title,
            &payload.content,
            path,
            payload.author.as_deref(),
            payload.comment.as_deref(),
        )
        .await
    {
        Ok(recipe) => {
            let recipe_id = generate_recipe_id(&recipe.git_path);
            Ok((
                StatusCode::CREATED,
                Json(RecipeResponse {
                    recipe_id,
                    recipe_name: recipe.name,
                    path: recipe.category,
                    file_name: recipe.file_name,
                    content: recipe.content,
                    description: recipe.description,
                }),
            ))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "creation_error",
                format!("Failed to create recipe: {}", e),
            )),
        )),
    }
}

/// List all recipes with pagination
pub async fn list_recipes(
    State(repo): State<Arc<RecipeRepository>>,
    Query(params): Query<ListQuery>,
) -> Json<RecipeListResponse> {
    let limit = std::cmp::min(params.limit.unwrap_or(20), 100);
    let offset = params.offset.unwrap_or(0);

    let all_recipes = repo.list_all();
    let total = all_recipes.len() as u32;

    let recipes: Vec<RecipeSummary> = all_recipes
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|recipe| {
            let recipe_id = generate_recipe_id(&recipe.git_path);
            RecipeSummary {
                recipe_id,
                recipe_name: recipe.name,
                path: recipe.category,
            }
        })
        .collect();

    Json(RecipeListResponse {
        recipes,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
        },
    })
}

/// Search recipes by name
pub async fn search_recipes(
    State(repo): State<Arc<RecipeRepository>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<RecipeListResponse>, (StatusCode, Json<ErrorResponse>)> {
    if params.q.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "Search query cannot be empty",
            )),
        ));
    }

    let limit = std::cmp::min(params.limit.unwrap_or(20), 100);
    let offset = params.offset.unwrap_or(0);

    let all_results = repo.search_by_name(&params.q);
    let total = all_results.len() as u32;

    let recipes: Vec<RecipeSummary> = all_results
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|recipe| {
            let recipe_id = generate_recipe_id(&recipe.git_path);
            RecipeSummary {
                recipe_id,
                recipe_name: recipe.name,
                path: recipe.category,
            }
        })
        .collect();

    Ok(Json(RecipeListResponse {
        recipes,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
        },
    }))
}

/// Get a single recipe by recipe_id
pub async fn get_recipe(
    State(repo): State<Arc<RecipeRepository>>,
    Path(recipe_id): Path<String>,
) -> Result<Json<RecipeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Look up git_path from recipe_id using the cache
    let git_path = repo.get_recipe_git_path(&recipe_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Recipe not found")),
        )
    })?;

    match repo.read(&git_path).await {
        Ok(recipe) => Ok(Json(RecipeResponse {
            recipe_id,
            recipe_name: recipe.name,
            path: recipe.category,
            file_name: recipe.file_name,
            content: recipe.content,
            description: recipe.description,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "read_error",
                format!("Failed to read recipe: {}", e),
            )),
        )),
    }
}

/// Update a recipe
pub async fn update_recipe(
    State(repo): State<Arc<RecipeRepository>>,
    Path(recipe_id): Path<String>,
    Json(payload): Json<UpdateRecipeRequest>,
) -> Result<Json<RecipeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate at least one field is provided
    if payload.content.is_none() && payload.path.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "At least one of 'content' or 'path' must be provided",
            )),
        ));
    }

    // Look up git_path from recipe_id
    let git_path = repo.get_recipe_git_path(&recipe_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Recipe not found")),
        )
    })?;

    // If content provided, validate it has YAML front matter with title
    if let Some(ref content) = payload.content {
        if extract_recipe_title(content).is_err() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "validation_error",
                    "Recipe content must include YAML front matter with 'title' field",
                )),
            ));
        }
    }

    // Convert empty path string to None
    let path = payload
        .path
        .as_deref()
        .and_then(|p| if p.trim().is_empty() { None } else { Some(p) });

    match repo
        .update_with_author_and_comment(
            &git_path,
            None, // name parameter deprecated (extracted from content)
            payload.content.as_deref(),
            path.map(Some),
            payload.author.as_deref(),
            payload.comment.as_deref(),
        )
        .await
    {
        Ok(recipe) => {
            let updated_id = generate_recipe_id(&recipe.git_path);
            Ok(Json(RecipeResponse {
                recipe_id: updated_id,
                recipe_name: recipe.name,
                path: recipe.category,
                file_name: recipe.file_name,
                content: recipe.content,
                description: recipe.description,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "update_error",
                format!("Failed to update recipe: {}", e),
            )),
        )),
    }
}

/// Delete a recipe
pub async fn delete_recipe(
    State(repo): State<Arc<RecipeRepository>>,
    Path(recipe_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Look up git_path from recipe_id
    let git_path = repo.get_recipe_git_path(&recipe_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Recipe not found")),
        )
    })?;

    match repo.delete(&git_path).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "delete_error",
                format!("Failed to delete recipe: {}", e),
            )),
        )),
    }
}

/// Find recipes by name (fallback lookup for when IDs change)
pub async fn find_recipe_by_name(
    State(repo): State<Arc<RecipeRepository>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<RecipeListResponse>, (StatusCode, Json<ErrorResponse>)> {
    if params.q.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "Search query cannot be empty",
            )),
        ));
    }

    let limit = std::cmp::min(params.limit.unwrap_or(20), 100);
    let offset = params.offset.unwrap_or(0);

    let all_results = repo.search_by_name(&params.q);
    let total = all_results.len() as u32;

    let recipes: Vec<RecipeSummary> = all_results
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|recipe| {
            let recipe_id = generate_recipe_id(&recipe.git_path);
            RecipeSummary {
                recipe_id,
                recipe_name: recipe.name,
                path: recipe.category,
            }
        })
        .collect();

    Ok(Json(RecipeListResponse {
        recipes,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
        },
    }))
}

/// Find a recipe by exact path (fallback lookup for when IDs change)
#[derive(serde::Deserialize)]
pub struct FindByPathQuery {
    pub path: Option<String>,
}

pub async fn find_recipe_by_path(
    State(repo): State<Arc<RecipeRepository>>,
    Query(params): Query<FindByPathQuery>,
) -> Result<Json<Vec<RecipeSummary>>, (StatusCode, Json<ErrorResponse>)> {
    let path = params.path.as_deref().unwrap_or("");

    // Find all recipes at the specified path
    let all_recipes = repo.list_all();

    let matching: Vec<RecipeSummary> = all_recipes
        .into_iter()
        .filter(|recipe| recipe.category.as_deref().unwrap_or("") == path)
        .map(|recipe| {
            let recipe_id = generate_recipe_id(&recipe.git_path);
            RecipeSummary {
                recipe_id,
                recipe_name: recipe.name,
                path: recipe.category,
            }
        })
        .collect();

    Ok(Json(matching))
}

/// List all categories
pub async fn list_categories(
    State(repo): State<Arc<RecipeRepository>>,
) -> Json<CategoryListResponse> {
    let categories = repo.get_categories();
    Json(CategoryListResponse { categories })
}

/// Get recipes in a category
pub async fn get_category_recipes(
    State(repo): State<Arc<RecipeRepository>>,
    Path(category_name): Path<String>,
) -> Result<Json<CategoryRecipesResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify category exists
    let categories = repo.get_categories();
    if !categories.contains(&category_name) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                format!("Path '{}' not found", category_name),
            )),
        ));
    }

    let recipes = repo.list_by_category(&category_name);
    let summaries: Vec<RecipeSummary> = recipes
        .into_iter()
        .map(|recipe| {
            let recipe_id = generate_recipe_id(&recipe.git_path);
            RecipeSummary {
                recipe_id,
                recipe_name: recipe.name,
                path: recipe.category,
            }
        })
        .collect();

    let count = summaries.len();

    Ok(Json(CategoryRecipesResponse {
        path: category_name,
        recipes: summaries,
        count,
    }))
}
