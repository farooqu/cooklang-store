use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::models::PaginationInfo;

/// Single recipe response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeResponse {
    /// Unique recipe ID (derived from git_path)
    pub recipe_id: String,
    /// Recipe name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional category
    pub category: Option<String>,
    /// Full recipe content in CookLang format
    pub content: String,
}

/// Recipe summary (without full content, for listings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    /// Unique recipe ID
    pub recipe_id: String,
    /// Recipe name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional category
    pub category: Option<String>,
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

/// Category recipes response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecipesResponse {
    pub category: String,
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

    pub fn with_details(
        mut self,
        details: HashMap<String, String>,
    ) -> Self {
        self.details = Some(details);
        self
    }
}
