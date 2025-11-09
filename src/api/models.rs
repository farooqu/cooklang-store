use serde::{Deserialize, Serialize};

/// Request body for creating a recipe
///
/// - `content`: required, must include YAML front matter with `title` field
/// - `path`: optional directory path (no `recipes/` prefix, defaults to root)
/// - `author`: optional git commit author
/// - `comment`: optional git commit message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    /// Recipe content in Cooklang format (must include YAML front matter with `title` field)
    pub content: String,
    /// Optional directory path (relative to data-dir, no `recipes/` prefix)
    pub path: Option<String>,
    /// Optional author name for git commit
    pub author: Option<String>,
    /// Optional comment for git commit
    pub comment: Option<String>,
}

/// Request body for updating a recipe
///
/// At least one of `content` or `path` must be provided
///
/// - `content`: optional new recipe content (must include YAML front matter with `title` if provided)
/// - `path`: optional new directory path (no `recipes/` prefix)
/// - `author`: optional git commit author
/// - `comment`: optional git commit message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecipeRequest {
    /// Optional new recipe content (must include YAML front matter with `title` if provided)
    pub content: Option<String>,
    /// Optional new directory path (relative to data-dir, no `recipes/` prefix)
    pub path: Option<String>,
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
