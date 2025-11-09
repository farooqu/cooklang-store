use cooklang_backend::{api, repository::RecipeRepository};
use std::sync::Arc;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// TEST SETUP & REQUEST BUILDING
// ============================================================================

pub async fn setup_api_with_storage(
    storage_type: &str,
) -> (impl Fn() -> axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo = RecipeRepository::with_storage(temp_dir.path(), storage_type)
        .await
        .expect("Failed to create repo");

    let repo_arc = Arc::new(repo);

    let build_router = move || api::build_router(repo_arc.clone());

    (build_router, temp_dir)
}

pub fn make_request(
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

pub async fn extract_response_body(response: axum::http::Response<axum::body::Body>) -> String {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body_bytes.to_vec()).unwrap()
}

// ============================================================================
// GIT REPOSITORY VERIFICATION HELPERS
// ============================================================================

pub fn verify_recipe_file_exists(temp_dir: &TempDir, recipe_name: &str, category: &str) -> String {
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

pub fn read_recipe_file(temp_dir: &TempDir, recipe_name: &str, category: &str) -> String {
    let filename = verify_recipe_file_exists(temp_dir, recipe_name, category);
    let path = temp_dir.path().join("recipes").join(category).join(&filename);
    fs::read_to_string(&path).expect("Failed to read recipe file")
}

pub fn count_git_commits(temp_dir: &TempDir) -> usize {
    let repo = git2::Repository::open(temp_dir.path()).expect("Failed to open git repo");
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk.count()
}

pub fn verify_recipe_file_deleted(temp_dir: &TempDir, recipe_name: &str, category: &str) {
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
