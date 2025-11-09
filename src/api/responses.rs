use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::models::PaginationInfo;

/// Single recipe response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeResponse {
    /// Unique recipe ID (derived from git_path)
    #[serde(rename = "recipeId")]
    pub recipe_id: String,
    /// Recipe name (derived from Cooklang YAML front matter)
    #[serde(rename = "recipeName")]
    pub recipe_name: String,
    /// Directory path (relative to data-dir, no `recipes/` prefix)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// File name on disk (derived from recipe title)
    #[serde(rename = "fileName")]
    pub file_name: String,
    /// Full recipe content in Cooklang format
    pub content: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Recipe summary (without full content, for listings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    /// Unique recipe ID
    #[serde(rename = "recipeId")]
    pub recipe_id: String,
    /// Recipe name (derived from Cooklang YAML front matter)
    #[serde(rename = "recipeName")]
    pub recipe_name: String,
    /// Directory path (relative to data-dir, no `recipes/` prefix)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Paginated list of recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeListResponse {
    pub recipes: Vec<RecipeSummary>,
    pub pagination: PaginationInfo,
}

/// Category list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryListResponse {
    pub categories: Vec<String>,
}

/// Category recipes response (deprecated - for backwards compatibility during transition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecipesResponse {
    pub path: String,
    pub recipes: Vec<RecipeSummary>,
    pub count: usize,
}

/// Status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub recipe_count: usize,
    pub categories: usize,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }
}
