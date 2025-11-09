use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

use crate::cache::{generate_recipe_id, CachedRecipe, RecipeIndex};
use crate::parser::parse_recipe;
use crate::storage::RecipeStorage;

/// Represents the structure of a recipe (for API and display)
#[derive(Debug, Clone)]
pub struct Recipe {
    pub git_path: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub content: String,
}

/// Manages recipe operations across storage backend and in-memory cache
pub struct RecipeRepository {
    cache: RecipeIndex,
    storage: Box<dyn RecipeStorage>,
}

impl RecipeRepository {
    /// Create a new recipe repository with the default storage backend (disk)
    pub async fn new(repo_path: &Path) -> Result<Self> {
        Self::with_storage(repo_path, "disk").await
    }

    /// Create a new recipe repository with a specific storage backend
    pub async fn with_storage(repo_path: &Path, storage_type: &str) -> Result<Self> {
        let storage = crate::storage::create_storage(storage_type, repo_path).await?;
        let cache = RecipeIndex::new();

        let repo = RecipeRepository { cache, storage };

        // Rebuild cache from storage on initialization
        repo.rebuild_from_storage().await?;

        Ok(repo)
    }

    /// Rebuild the entire cache from storage files
    pub async fn rebuild_from_storage(&self) -> Result<()> {
        self.cache.clear();

        let cook_files = self.storage.discover_files()?;

        for git_path in cook_files {
            // Read the file content
            match self.storage.read_file(&git_path) {
                Ok(content) => {
                    // Extract category from path (recipes/{category}/{...}/{slug}.cook)
                    let category = self.extract_category_from_path(&git_path);

                    // Try to parse the recipe
                    let name_from_path = self.path_to_name(&git_path);
                    match parse_recipe(&content, &name_from_path) {
                        Ok(parsed_recipe) => {
                            let recipe_id = generate_recipe_id(&git_path);
                            let cached = CachedRecipe {
                                recipe_id,
                                git_path: git_path.clone(),
                                name: parsed_recipe.name.clone(),
                                description: None,
                                category,
                                recipe: parsed_recipe,
                            };
                            self.cache.insert(git_path, cached);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse recipe {}: {}", git_path, e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read recipe file {}: {}", git_path, e);
                }
            }
        }

        Ok(())
    }

    /// Create a new recipe
    pub async fn create(
        &self,
        name: &str,
        content: &str,
        category: Option<&str>,
    ) -> Result<Recipe> {
        self.create_with_author_and_comment(name, content, category, None, None)
            .await
    }

    /// Create a new recipe with an optional author
    pub async fn create_with_author(
        &self,
        name: &str,
        content: &str,
        category: Option<&str>,
        author: Option<&str>,
    ) -> Result<Recipe> {
        self.create_with_author_and_comment(name, content, category, author, None)
            .await
    }

    /// Create a new recipe with optional author and comment
    pub async fn create_with_author_and_comment(
        &self,
        name: &str,
        content: &str,
        category: Option<&str>,
        _author: Option<&str>,
        _comment: Option<&str>,
    ) -> Result<Recipe> {
        // Parse the recipe to validate it
        parse_recipe(content, name).map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;

        // Generate path from name and category
        let git_path = self.generate_git_path(name, category).await?;

        // Write to storage (source of truth)
        self.storage.write_file(&git_path, content)?;

        // Update cache
        let parsed =
            parse_recipe(content, name).map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;

        let recipe_id = generate_recipe_id(&git_path);
        let cached = CachedRecipe {
            recipe_id,
            git_path: git_path.clone(),
            name: name.to_string(),
            description: None,
            category: category.map(|s| s.to_string()),
            recipe: parsed,
        };

        self.cache.insert(git_path.clone(), cached);

        Ok(Recipe {
            git_path,
            name: name.to_string(),
            description: None,
            category: category.map(|s| s.to_string()),
            content: content.to_string(),
        })
    }

    /// Read a recipe by git path
    pub async fn read(&self, git_path: &str) -> Result<Recipe> {
        let cached = self
            .cache
            .get(git_path)
            .ok_or_else(|| anyhow!("Recipe not found: {}", git_path))?;

        let content = self.storage.read_file(git_path)?;

        Ok(Recipe {
            git_path: cached.git_path,
            name: cached.name,
            description: cached.description,
            category: cached.category,
            content,
        })
    }

    /// Update a recipe
    pub async fn update(
        &self,
        git_path: &str,
        name: Option<&str>,
        content: Option<&str>,
        category: Option<Option<&str>>,
    ) -> Result<Recipe> {
        self.update_with_author_and_comment(git_path, name, content, category, None, None)
            .await
    }

    /// Update a recipe with an optional author
    pub async fn update_with_author(
        &self,
        git_path: &str,
        name: Option<&str>,
        content: Option<&str>,
        category: Option<Option<&str>>,
        author: Option<&str>,
    ) -> Result<Recipe> {
        self.update_with_author_and_comment(git_path, name, content, category, author, None)
            .await
    }

    /// Update a recipe with optional author and comment
    pub async fn update_with_author_and_comment(
        &self,
        git_path: &str,
        name: Option<&str>,
        content: Option<&str>,
        category: Option<Option<&str>>,
        _author: Option<&str>,
        _comment: Option<&str>,
    ) -> Result<Recipe> {
        // Read current recipe from cache
        let current = self
            .cache
            .get(git_path)
            .ok_or_else(|| anyhow!("Recipe not found: {}", git_path))?;

        // Prepare new values
        let new_name = name.unwrap_or(&current.name);
        let new_category = category
            .as_ref()
            .copied()
            .flatten()
            .or(current.category.as_deref());

        // Validate new content if provided
        if let Some(c) = content {
            parse_recipe(c, new_name).map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;
        }

        // If name or category changed, update path
        let new_git_path = if name.is_some() || category.is_some() {
            self.generate_git_path(new_name, new_category).await?
        } else {
            git_path.to_string()
        };

        // Write to storage (if content provided or path changed)
        if content.is_some() || new_git_path != git_path {
            // Write content (use new content if provided, otherwise read current)
            let file_content = if let Some(c) = content {
                c.to_string()
            } else {
                self.storage.read_file(git_path)?
            };

            self.storage.write_file(&new_git_path, &file_content)?;

            // If path changed, delete old file
            if new_git_path != git_path {
                self.storage.delete_file(git_path)?;
            }
        }

        // Update cache
        let file_content = self.storage.read_file(&new_git_path)?;
        let parsed = parse_recipe(&file_content, new_name)
            .map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;

        if new_git_path != git_path {
            self.cache.remove(git_path);
        }

        let recipe_id = generate_recipe_id(&new_git_path);
        let cached = CachedRecipe {
            recipe_id,
            git_path: new_git_path.clone(),
            name: new_name.to_string(),
            description: None,
            category: new_category.map(|s| s.to_string()),
            recipe: parsed,
        };

        self.cache.insert(new_git_path.clone(), cached);

        Ok(Recipe {
            git_path: new_git_path,
            name: new_name.to_string(),
            description: None,
            category: new_category.map(|s| s.to_string()),
            content: file_content,
        })
    }

    /// Delete a recipe
    pub async fn delete(&self, git_path: &str) -> Result<()> {
        self.delete_with_author_and_comment(git_path, None, None)
            .await
    }

    /// Delete a recipe with an optional author
    pub async fn delete_with_author(&self, git_path: &str, author: Option<&str>) -> Result<()> {
        self.delete_with_author_and_comment(git_path, author, None)
            .await
    }

    /// Delete a recipe with optional author and comment
    pub async fn delete_with_author_and_comment(
        &self,
        git_path: &str,
        _author: Option<&str>,
        _comment: Option<&str>,
    ) -> Result<()> {
        // Verify recipe exists in cache
        let _cached = self
            .cache
            .get(git_path)
            .ok_or_else(|| anyhow!("Recipe not found: {}", git_path))?;

        // Delete from storage
        self.storage.delete_file(git_path)?;

        // Delete from cache
        self.cache.remove(git_path);

        Ok(())
    }

    /// List all recipes
    pub fn list_all(&self) -> Vec<Recipe> {
        self.cache
            .get_all()
            .into_iter()
            .map(|cached| Recipe {
                git_path: cached.git_path,
                name: cached.name,
                description: cached.description,
                category: cached.category,
                content: String::new(), // Content not included in list
            })
            .collect()
    }

    /// Search recipes by name
    pub fn search_by_name(&self, query: &str) -> Vec<Recipe> {
        self.cache
            .search_by_name(query)
            .into_iter()
            .map(|cached| Recipe {
                git_path: cached.git_path,
                name: cached.name,
                description: cached.description,
                category: cached.category,
                content: String::new(),
            })
            .collect()
    }

    /// Get recipes by category
    pub fn list_by_category(&self, category: &str) -> Vec<Recipe> {
        self.cache
            .get_by_category(category)
            .into_iter()
            .map(|cached| Recipe {
                git_path: cached.git_path,
                name: cached.name,
                description: cached.description,
                category: cached.category,
                content: String::new(),
            })
            .collect()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        self.cache.get_categories()
    }

    /// Get git_path by recipe_id
    pub fn get_recipe_git_path(&self, recipe_id: &str) -> Option<String> {
        self.cache.get_git_path(recipe_id)
    }

    /// Generate a path from recipe name and category
    async fn generate_git_path(&self, name: &str, category: Option<&str>) -> Result<String> {
        let slug = self.name_to_slug(name);
        let mut path = if let Some(cat) = category {
            format!("recipes/{}/{}.cook", cat, slug)
        } else {
            format!("recipes/{}.cook", slug)
        };

        // Check for duplicates and append numeric suffix if needed
        let mut counter = 2;
        while self.cache.get(&path).is_some() {
            path = if let Some(cat) = category {
                format!("recipes/{}/{}-{}.cook", cat, slug, counter)
            } else {
                format!("recipes/{}-{}.cook", slug, counter)
            };
            counter += 1;
        }

        Ok(path)
    }

    /// Convert recipe name to URL-friendly slug
    fn name_to_slug(&self, name: &str) -> String {
        lazy_static! {
            static ref SLUG_RE: Regex = Regex::new(r"[^a-z0-9]+").unwrap();
        }

        let slug = name.to_lowercase();
        let slug = SLUG_RE.replace_all(&slug, "-");
        slug.trim_matches('-').to_string()
    }

    /// Extract category from git path
    fn extract_category_from_path(&self, git_path: &str) -> Option<String> {
        // Expected: recipes/{category}/{...}/{slug}.cook
        let parts: Vec<&str> = git_path.split('/').collect();
        if parts.len() >= 3 && parts[0] == "recipes" {
            Some(parts[1].to_string())
        } else {
            None
        }
    }

    /// Convert git path to recipe name
    fn path_to_name(&self, git_path: &str) -> String {
        // Extract filename without extension
        git_path
            .split('/')
            .next_back()
            .and_then(|f| f.strip_suffix(".cook"))
            .unwrap_or("")
            .replace('-', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_repo() -> Result<(RecipeRepository, TempDir)> {
        let git_dir = TempDir::new()?;
        let repo = RecipeRepository::new(git_dir.path()).await?; // defaults to disk
        Ok((repo, git_dir))
    }

    async fn setup_git_test_repo() -> Result<(RecipeRepository, TempDir)> {
        let git_dir = TempDir::new()?;
        let repo = RecipeRepository::with_storage(git_dir.path(), "git").await?;
        Ok((repo, git_dir))
    }

    #[tokio::test]
    async fn test_create_recipe() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Simple Recipe\n\n@ingredient{} Some ingredient";
        let recipe = repo
            .create("Simple Recipe", content, Some("desserts"))
            .await?;

        assert_eq!(recipe.name, "Simple Recipe");
        assert_eq!(recipe.category, Some("desserts".to_string()));
        assert!(recipe.git_path.contains("simple-recipe"));
        assert!(recipe.git_path.ends_with(".cook"));

        // Verify it's in cache
        assert_eq!(repo.cache.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_read_recipe() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{} test";
        let created = repo.create("Test Recipe", content, None).await?;
        let read = repo.read(&created.git_path).await?;

        assert_eq!(read.name, created.name);
        assert_eq!(read.git_path, created.git_path);
        assert_eq!(read.content, content);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_recipe() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        assert_eq!(repo.cache.len(), 1);
        repo.delete(&recipe.git_path).await?;
        assert_eq!(repo.cache.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_with_author_disk() -> Result<()> {
        // Author parameter is accepted but doesn't affect disk storage
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{} test";
        let recipe = repo
            .create_with_author("Test Recipe", content, Some("desserts"), Some("Alice"))
            .await?;

        assert_eq!(recipe.name, "Test Recipe");
        assert_eq!(recipe.category, Some("desserts".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_with_author_git() -> Result<()> {
        // Git storage creates commits with author information
        let (repo, _git) = setup_git_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{} test";
        let recipe = repo
            .create_with_author("Test Recipe", content, Some("desserts"), Some("Alice"))
            .await?;

        assert_eq!(recipe.name, "Test Recipe");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_with_author() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        let new_content = "# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author(&recipe.git_path, None, Some(new_content), None, Some("Bob"))
            .await?;

        let updated = repo.read(&recipe.git_path).await?;
        assert_eq!(updated.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_with_author() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        repo.delete_with_author(&recipe.git_path, Some("Charlie"))
            .await?;

        // Verify it's deleted
        assert!(repo.read(&recipe.git_path).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_with_author_and_comment() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{} test";
        let recipe = repo
            .create_with_author_and_comment(
                "Test Recipe",
                content,
                Some("desserts"),
                Some("Alice"),
                Some("Added classic chocolate cake"),
            )
            .await?;

        assert_eq!(recipe.name, "Test Recipe");
        assert_eq!(recipe.category, Some("desserts".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_with_author_and_comment() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        let new_content = "# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author_and_comment(
            &recipe.git_path,
            None,
            Some(new_content),
            None,
            Some("Bob"),
            Some("Fixed ingredient quantities"),
        )
        .await?;

        let updated = repo.read(&recipe.git_path).await?;
        assert_eq!(updated.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_content_only() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author(
            &recipe.git_path,
            None,
            Some(new_content),
            None,
            Some("Alice"),
        )
        .await?;

        let updated = repo.read(&recipe.git_path).await?;
        assert_eq!(updated.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_rename_only() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        repo.update_with_author(&recipe.git_path, Some("New Name"), None, None, Some("Bob"))
            .await?;

        let all = repo.list_all();
        assert!(all.iter().any(|r| r.name == "New Name"));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_move_only() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        repo.update_with_author(
            &recipe.git_path,
            None,
            None,
            Some(Some("mains")),
            Some("Charlie"),
        )
        .await?;

        let mains = repo.list_by_category("mains");
        assert_eq!(mains.len(), 1);
        assert_eq!(mains[0].name, "Test");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_rename_and_move() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        repo.update_with_author(
            &recipe.git_path,
            Some("New Name"),
            None,
            Some(Some("mains")),
            Some("Alice"),
        )
        .await?;

        let mains = repo.list_by_category("mains");
        assert_eq!(mains.len(), 1);
        assert_eq!(mains[0].name, "New Name");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_content_and_rename() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author(
            &recipe.git_path,
            Some("New Name"),
            Some(new_content),
            None,
            Some("Bob"),
        )
        .await?;

        let all = repo.list_all();
        let updated = all.iter().find(|r| r.name == "New Name").unwrap();
        let recipe = repo.read(&updated.git_path).await?;
        assert_eq!(recipe.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_content_and_recategorize() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author(
            &recipe.git_path,
            None,
            Some(new_content),
            Some(Some("mains")),
            Some("Charlie"),
        )
        .await?;

        let mains = repo.list_by_category("mains");
        assert_eq!(mains.len(), 1);
        let recipe = repo.read(&mains[0].git_path).await?;
        assert_eq!(recipe.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_all_three_changes() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author_and_comment(
            &recipe.git_path,
            Some("New Name"),
            Some(new_content),
            Some(Some("mains")),
            Some("Alice"),
            Some("Complete overhaul"),
        )
        .await?;

        let mains = repo.list_by_category("mains");
        assert_eq!(mains.len(), 1);
        assert_eq!(mains[0].name, "New Name");
        let recipe = repo.read(&mains[0].git_path).await?;
        assert_eq!(recipe.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_with_author_and_comment() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        repo.delete_with_author_and_comment(
            &recipe.git_path,
            Some("Charlie"),
            Some("Duplicate recipe"),
        )
        .await?;

        assert!(repo.read(&recipe.git_path).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_name() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        repo.create(
            "Chocolate Cake",
            "# Chocolate\n\n@ingredient{}",
            Some("desserts"),
        )
        .await?;
        repo.create(
            "Vanilla Cake",
            "# Vanilla\n\n@ingredient{}",
            Some("desserts"),
        )
        .await?;
        repo.create("Pasta", "# Pasta\n\n@ingredient{}", Some("mains"))
            .await?;

        let results = repo.search_by_name("cake");
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_by_category() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        repo.create("Cake", "# Cake\n\n@ingredient{}", Some("desserts"))
            .await?;
        repo.create("Brownie", "# Brownie\n\n@ingredient{}", Some("desserts"))
            .await?;
        repo.create("Pasta", "# Pasta\n\n@ingredient{}", Some("mains"))
            .await?;

        let desserts = repo.list_by_category("desserts");
        assert_eq!(desserts.len(), 2);

        let mains = repo.list_by_category("mains");
        assert_eq!(mains.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_name_to_slug() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo = RecipeRepository::new(temp_dir.path()).await?;

        assert_eq!(repo.name_to_slug("Simple Recipe"), "simple-recipe");
        assert_eq!(
            repo.name_to_slug("Triple-Chocolate Cake!"),
            "triple-chocolate-cake"
        );
        assert_eq!(repo.name_to_slug("CamelCaseRecipe"), "camelcaserecipe");

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_category_from_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo = RecipeRepository::new(temp_dir.path()).await?;

        assert_eq!(
            repo.extract_category_from_path("recipes/desserts/cake.cook"),
            Some("desserts".to_string())
        );
        assert_eq!(
            repo.extract_category_from_path("recipes/desserts/chocolate/cake.cook"),
            Some("desserts".to_string())
        );
        assert_eq!(repo.extract_category_from_path("recipes/cake.cook"), None);

        Ok(())
    }

    #[tokio::test]
    async fn test_path_to_name() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo = RecipeRepository::new(temp_dir.path()).await?;

        assert_eq!(
            repo.path_to_name("recipes/simple-recipe.cook"),
            "Simple Recipe"
        );
        assert_eq!(
            repo.path_to_name("recipes/desserts/chocolate-cake.cook"),
            "Chocolate Cake"
        );

        Ok(())
    }
}
