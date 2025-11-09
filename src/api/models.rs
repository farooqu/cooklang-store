use serde::{Deserialize, Serialize};

/// Request body for creating a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    /// Recipe name
    pub name: String,
    /// Recipe content in Cooklang format
    pub content: String,
    /// Optional category for the recipe
    pub category: Option<String>,
    /// Optional author name for git commit
    pub author: Option<String>,
    /// Optional comment for git commit
    pub comment: Option<String>,
}

/// Request body for updating a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecipeRequest {
    /// Optional new recipe name
    pub name: Option<String>,
    /// Optional new recipe content
    pub content: Option<String>,
    /// Optional new category (use null to remove category)
    #[serde(default)]
    pub category: Option<Option<String>>,
    /// Optional author name for git commit
    pub author: Option<String>,
    /// Optional comment for git commit
    pub comment: Option<String>,
}

/// Query parameters for listing recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuery {
    /// Number of items per page (default: 20, max: 100)
    pub limit: Option<u32>,
    /// Number of items to skip (default: 0)
    pub offset: Option<u32>,
}

/// Query parameters for searching recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search query term
    pub q: String,
    /// Number of items per page (default: 20, max: 100)
    pub limit: Option<u32>,
    /// Number of items to skip (default: 0)
    pub offset: Option<u32>,
}

/// Pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
}
