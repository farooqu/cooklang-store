use cooklang_backend::{api, repository::RecipeRepository};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// FIXTURE LOADING & SEEDING
// ============================================================================

pub fn load_recipe_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}.cook", name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to load recipe fixture: {}", path))
}

/// Copy a fixture file from tests/fixtures/ to the temp directory's recipes folder.
/// 
/// This is useful for tests that need pre-existing recipe files on disk.
/// The fixture content is read from `tests/fixtures/{fixture_name}.cook` and written to
/// the temp directory at `recipes/{category}/{filename}`.
/// 
/// # Arguments
/// - `temp_dir`: The temporary directory created for the test (from setup_api_with_storage)
/// - `fixture_name`: The name of the fixture file (without .cook extension)
/// - `category`: Optional category path (e.g., "desserts/cakes" for nested categories)
/// - `filename`: The filename to write in the recipes directory (e.g., "my-recipe.cook")
/// 
/// # Example
/// ```ignore
/// let (build_router, temp_dir) = setup_api_with_storage("git").await;
/// // Seed a pre-existing recipe file to test loading behavior
/// copy_fixture_to_recipes_dir(&temp_dir, "test-recipe", Some("desserts"), "test-recipe.cook");
/// // Now verify it exists or perform operations on it
/// ```
pub fn copy_fixture_to_recipes_dir(temp_dir: &TempDir, fixture_name: &str, category: Option<&str>, filename: &str) {
    let fixture_path = format!("tests/fixtures/{}.cook", fixture_name);
    let content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to load recipe fixture: {}", fixture_path));
    
    let recipes_dir = if let Some(cat) = category {
        let mut path = temp_dir.path().join("recipes");
        for part in cat.split('/') {
            path = path.join(part);
        }
        path
    } else {
        temp_dir.path().join("recipes")
    };
    
    fs::create_dir_all(&recipes_dir).expect("Failed to create recipes directory");
    
    let file_path = recipes_dir.join(filename);
    fs::write(&file_path, &content).expect("Failed to write fixture file to temp directory");
}

// ============================================================================
// TEST SETUP & REQUEST BUILDING
// ============================================================================

pub async fn setup_api_with_storage(storage_type: &str) -> (impl Fn() -> axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo = RecipeRepository::with_storage(temp_dir.path(), storage_type)
        .await
        .expect("Failed to create repo");

    let repo_arc = Arc::new(repo);

    let build_router = move || api::build_router(repo_arc.clone());

    (build_router, temp_dir)
}

/// Setup API with pre-seeded fixture files.
/// 
/// Use this when tests need pre-existing recipe files on disk.
/// This creates the temp directory, seeds the specified fixtures, then initializes the repository.
/// 
/// # Example
/// ```ignore
/// let (build_router, temp_dir) = setup_api_with_seeded_fixtures("git", vec![
///     ("recipe-1", Some("desserts"), "recipe-1.cook"),
///     ("recipe-2", Some("desserts"), "recipe-2.cook"),
/// ]).await;
/// ```
pub async fn setup_api_with_seeded_fixtures(
    storage_type: &str,
    fixtures: Vec<(&str, Option<&str>, &str)>,
) -> (impl Fn() -> axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    
    // Seed all fixtures first
    for (fixture_name, category, filename) in fixtures {
        copy_fixture_to_recipes_dir(&temp_dir, fixture_name, category, filename);
    }
    
    // Now create repository - it will load the seeded files into cache
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

    if let Some(json_body) = body {
        builder = builder.header("content-type", "application/json");
        builder
            .body(axum::body::Body::from(json_body.to_string()))
            .unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
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
    let recipes_dir = if category.contains('/') {
        // Handle nested categories: "meals/meat/traditional"
        let mut path = temp_dir.path().join("recipes");
        for part in category.split('/') {
            path = path.join(part);
        }
        path
    } else {
        // Single-level category
        temp_dir.path().join("recipes").join(category)
    };

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
    let path = if category.contains('/') {
        // Handle nested categories: "meals/meat/traditional"
        let mut path = temp_dir.path().join("recipes");
        for part in category.split('/') {
            path = path.join(part);
        }
        path.join(&filename)
    } else {
        // Single-level category
        temp_dir.path().join("recipes").join(category).join(&filename)
    };
    fs::read_to_string(&path).expect("Failed to read recipe file")
}

pub fn verify_recipe_file_exists_at_root(temp_dir: &TempDir, recipe_name: &str) -> String {
    let recipes_dir = temp_dir.path().join("recipes");
    assert!(
        recipes_dir.exists(),
        "Recipes directory doesn't exist: {}",
        recipes_dir.display()
    );

    // Find the recipe file at root (could be recipe-name.cook or recipe-name-2.cook if duplicate)
    let name_slug = recipe_name.to_lowercase().replace(" ", "-");
    let files: Vec<_> = fs::read_dir(&recipes_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            // Only look for direct children (not in subdirectories)
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
        "Recipe file not found for '{}' at root of recipes directory",
        recipe_name
    );

    files[0].clone()
}

pub fn read_recipe_file_at_root(temp_dir: &TempDir, recipe_name: &str) -> String {
    let filename = verify_recipe_file_exists_at_root(temp_dir, recipe_name);
    let path = temp_dir.path().join("recipes").join(&filename);
    fs::read_to_string(&path).expect("Failed to read recipe file at root")
}

#[allow(dead_code)]
pub fn count_git_commits(temp_dir: &TempDir) -> usize {
    let repo = git2::Repository::open(temp_dir.path()).expect("Failed to open git repo");
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk.count()
}

pub fn verify_recipe_file_deleted(temp_dir: &TempDir, recipe_name: &str, category: &str) {
    let recipes_dir = if category.contains('/') {
        // Handle nested categories: "meals/meat/traditional"
        let mut path = temp_dir.path().join("recipes");
        for part in category.split('/') {
            path = path.join(part);
        }
        path
    } else {
        // Single-level category
        temp_dir.path().join("recipes").join(category)
    };

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
