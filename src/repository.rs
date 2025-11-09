use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

use crate::cache::{generate_recipe_id, CachedRecipe, RecipeIndex};
use crate::parser::{parse_recipe, extract_recipe_title, generate_filename, should_rename_file};
use crate::storage::RecipeStorage;

/// Represents the structure of a recipe (for API and display)
#[derive(Debug, Clone)]
pub struct Recipe {
    pub git_path: String,
    pub file_name: String,
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

                    // Try to extract title from YAML front matter
                    let recipe_name = match extract_recipe_title(&content) {
                        Ok(title) => title,
                        Err(_) => {
                            // Fallback to path-based name if YAML front matter missing
                            tracing::warn!("Recipe {} missing YAML front matter, using path-based name", git_path);
                            self.path_to_name(&git_path)
                        }
                    };

                    match parse_recipe(&content, &recipe_name) {
                        Ok(parsed_recipe) => {
                            let recipe_id = generate_recipe_id(&git_path);
                            let cached = CachedRecipe {
                                recipe_id,
                                git_path: git_path.clone(),
                                name: recipe_name.clone(),
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
        // Extract title from YAML front matter (content must have it)
        let recipe_title = extract_recipe_title(content)
            .map_err(|e| anyhow!("Invalid recipe content: {}", e))?;

        // Validate the recipe can be parsed
        parse_recipe(content, &recipe_title).map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;

        // Generate filename from the extracted title
        let filename = generate_filename(&recipe_title);

        // Generate path from filename and category
        let git_path = self.generate_git_path_from_filename(&filename, category).await?;

        // Write to storage (source of truth)
        self.storage.write_file(&git_path, content)?;

        // Update cache
        let parsed =
            parse_recipe(content, &recipe_title).map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;

        let recipe_id = generate_recipe_id(&git_path);
        let cached = CachedRecipe {
            recipe_id,
            git_path: git_path.clone(),
            name: recipe_title.clone(),
            description: None,
            category: category.map(|s| s.to_string()),
            recipe: parsed,
        };

        self.cache.insert(git_path.clone(), cached);

        Ok(Recipe {
            git_path: git_path.clone(),
            file_name: filename,
            name: recipe_title,
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
        let file_name = self.extract_filename_from_path(git_path);

        Ok(Recipe {
            git_path: cached.git_path,
            file_name,
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

        // Read current content from storage
        let current_content = self.storage.read_file(git_path)?;

        // Determine the new recipe title
        // Priority: extracted title from new content → provided name parameter → current name
        let new_title = if let Some(c) = content {
            // Extract title from new content if provided
            extract_recipe_title(c)
                .map_err(|e| anyhow!("Invalid recipe content: {}", e))?
        } else if let Some(n) = name {
            // Use provided name if content not changing
            n.to_string()
        } else {
            // Keep current title
            current.name.clone()
        };

        let new_category = category
            .as_ref()
            .copied()
            .flatten()
            .or(current.category.as_deref());

        // Validate new content if provided
        if let Some(c) = content {
            parse_recipe(c, &new_title).map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;
        }

        // Generate new filename from title
        let old_filename = self.extract_filename_from_path(git_path);
        let new_filename = generate_filename(&new_title);

        // Check if rename is needed (if filename changed or category changed)
        let filename_changed = should_rename_file(&old_filename, &new_title);
        let category_changed = new_category != current.category.as_deref();

        // Generate new git_path if anything changed
        let new_git_path = if filename_changed || category_changed {
            self.generate_git_path_from_filename(&new_filename, new_category).await?
        } else {
            git_path.to_string()
        };

        // Write to storage (if content provided or path changed)
        if content.is_some() || new_git_path != git_path {
            // Write content (use new content if provided, otherwise read current)
            let file_content = if let Some(c) = content {
                c.to_string()
            } else {
                current_content.clone()
            };

            self.storage.write_file(&new_git_path, &file_content)?;

            // If path changed, delete old file
            if new_git_path != git_path {
                self.storage.delete_file(git_path)?;
            }
        }

        // Update cache
        let file_content = self.storage.read_file(&new_git_path)?;
        let parsed = parse_recipe(&file_content, &new_title)
            .map_err(|e| anyhow!("Failed to parse recipe: {}", e))?;

        if new_git_path != git_path {
            self.cache.remove(git_path);
        }

        let recipe_id = generate_recipe_id(&new_git_path);
        let cached = CachedRecipe {
            recipe_id,
            git_path: new_git_path.clone(),
            name: new_title.clone(),
            description: None,
            category: new_category.map(|s| s.to_string()),
            recipe: parsed,
        };

        self.cache.insert(new_git_path.clone(), cached);

        Ok(Recipe {
            git_path: new_git_path,
            file_name: new_filename,
            name: new_title,
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
            .map(|cached| {
                let file_name = self.extract_filename_from_path(&cached.git_path);
                Recipe {
                    git_path: cached.git_path,
                    file_name,
                    name: cached.name,
                    description: cached.description,
                    category: cached.category,
                    content: String::new(), // Content not included in list
                }
            })
            .collect()
    }

    /// Search recipes by name
    pub fn search_by_name(&self, query: &str) -> Vec<Recipe> {
        self.cache
            .search_by_name(query)
            .into_iter()
            .map(|cached| {
                let file_name = self.extract_filename_from_path(&cached.git_path);
                Recipe {
                    git_path: cached.git_path,
                    file_name,
                    name: cached.name,
                    description: cached.description,
                    category: cached.category,
                    content: String::new(),
                }
            })
            .collect()
    }

    /// Get recipes by category
    pub fn list_by_category(&self, category: &str) -> Vec<Recipe> {
        self.cache
            .get_by_category(category)
            .into_iter()
            .map(|cached| {
                let file_name = self.extract_filename_from_path(&cached.git_path);
                Recipe {
                    git_path: cached.git_path,
                    file_name,
                    name: cached.name,
                    description: cached.description,
                    category: cached.category,
                    content: String::new(),
                }
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

    /// Generate a path from recipe name and category (deprecated, kept for backward compatibility)
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

    /// Generate a git path from a filename and category
    async fn generate_git_path_from_filename(&self, filename: &str, category: Option<&str>) -> Result<String> {
        let mut path = if let Some(cat) = category {
            format!("recipes/{}/{}", cat, filename)
        } else {
            format!("recipes/{}", filename)
        };

        // Check for duplicates and append numeric suffix if needed
        let mut counter = 2;
        while self.cache.get(&path).is_some() {
            // Insert counter before .cook extension
            let base = filename.strip_suffix(".cook").unwrap_or(filename);
            let new_filename = format!("{}-{}.cook", base, counter);
            path = if let Some(cat) = category {
                format!("recipes/{}/{}", cat, new_filename)
            } else {
                format!("recipes/{}", new_filename)
            };
            counter += 1;
        }

        Ok(path)
    }

    /// Extract filename from a git path
    fn extract_filename_from_path(&self, git_path: &str) -> String {
        git_path
            .split('/')
            .next_back()
            .unwrap_or("")
            .to_string()
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
    /// Categories support hierarchical nesting: recipes/meals/meat/traditional/chicken-biryani.cook
    /// would have category "meals/meat/traditional"
    fn extract_category_from_path(&self, git_path: &str) -> Option<String> {
        // Expected: recipes/{category/path}/{slug}.cook
        let parts: Vec<&str> = git_path.split('/').collect();
        if parts.len() >= 3 && parts[0] == "recipes" {
            // All parts between "recipes/" and the filename form the category path
            let category_parts = &parts[1..parts.len() - 1];
            if !category_parts.is_empty() {
                Some(category_parts.join("/"))
            } else {
                None
            }
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

        let content = "---\ntitle: Simple Recipe\n---\n\n# Simple Recipe\n\n@ingredient{} Some ingredient";
        let recipe = repo
            .create("Simple Recipe", content, Some("desserts"))
            .await?;

        assert_eq!(recipe.name, "Simple Recipe");
        assert_eq!(recipe.file_name, "simple-recipe.cook");
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

        let content = "---\ntitle: Test Recipe\n---\n\n# Test Recipe\n\n@ingredient{} test";
        let created = repo.create("Test Recipe", content, None).await?;
        let read = repo.read(&created.git_path).await?;

        assert_eq!(read.name, created.name);
        assert_eq!(read.git_path, created.git_path);
        assert_eq!(read.file_name, created.file_name);
        assert_eq!(read.content, content);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_recipe() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
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

        let content = "---\ntitle: Test Recipe\n---\n\n# Test Recipe\n\n@ingredient{} test";
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

        let content = "---\ntitle: Test Recipe\n---\n\n# Test Recipe\n\n@ingredient{} test";
        let recipe = repo
            .create_with_author("Test Recipe", content, Some("desserts"), Some("Alice"))
            .await?;

        assert_eq!(recipe.name, "Test Recipe");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_with_author() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        let new_content = "---\ntitle: Test\n---\n\n# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author(&recipe.git_path, None, Some(new_content), None, Some("Bob"))
            .await?;

        let updated = repo.read(&recipe.git_path).await?;
        assert_eq!(updated.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_with_author() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
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

        let content = "---\ntitle: Test Recipe\n---\n\n# Test Recipe\n\n@ingredient{} test";
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, None).await?;

        let new_content = "---\ntitle: Test\n---\n\n# Test Recipe Updated\n\n@ingredient{} updated";
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "---\ntitle: Test\n---\n\n# Test Recipe Updated\n\n@ingredient{} updated";
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "---\ntitle: New Name\n---\n\n# New Name\n\n@ingredient{}";
        repo.update_with_author(&recipe.git_path, None, Some(new_content), None, Some("Bob"))
            .await?;

        let all = repo.list_all();
        assert!(all.iter().any(|r| r.name == "New Name"));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_move_only() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "---\ntitle: New Name\n---\n\n# New Name\n\n@ingredient{}";
        repo.update_with_author(
            &recipe.git_path,
            None,
            Some(new_content),
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "---\ntitle: New Name\n---\n\n# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author(
            &recipe.git_path,
            None,
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "---\ntitle: Test\n---\n\n# Test Recipe Updated\n\n@ingredient{} updated";
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
        let recipe = repo.create("Test", content, Some("desserts")).await?;

        let new_content = "---\ntitle: New Name\n---\n\n# Test Recipe Updated\n\n@ingredient{} updated";
        repo.update_with_author_and_comment(
            &recipe.git_path,
            None,
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

        let content = "---\ntitle: Test\n---\n\n# Test Recipe\n\n@ingredient{}";
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
            "---\ntitle: Chocolate Cake\n---\n\n# Chocolate\n\n@ingredient{}",
            Some("desserts"),
        )
        .await?;
        repo.create(
            "Vanilla Cake",
            "---\ntitle: Vanilla Cake\n---\n\n# Vanilla\n\n@ingredient{}",
            Some("desserts"),
        )
        .await?;
        repo.create("Pasta", "---\ntitle: Pasta\n---\n\n# Pasta\n\n@ingredient{}", Some("mains"))
            .await?;

        let results = repo.search_by_name("cake");
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_by_category() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        repo.create("Cake", "---\ntitle: Cake\n---\n\n# Cake\n\n@ingredient{}", Some("desserts"))
            .await?;
        repo.create("Brownie", "---\ntitle: Brownie\n---\n\n# Brownie\n\n@ingredient{}", Some("desserts"))
            .await?;
        repo.create("Pasta", "---\ntitle: Pasta\n---\n\n# Pasta\n\n@ingredient{}", Some("mains"))
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

        // Single-level category
        assert_eq!(
            repo.extract_category_from_path("recipes/desserts/cake.cook"),
            Some("desserts".to_string())
        );
        // Hierarchical category (nested directories)
        assert_eq!(
            repo.extract_category_from_path("recipes/desserts/chocolate/cake.cook"),
            Some("desserts/chocolate".to_string())
        );
        // Deeper nesting
        assert_eq!(
            repo.extract_category_from_path("recipes/meals/meat/traditional/chicken-biryani.cook"),
            Some("meals/meat/traditional".to_string())
        );
        // Root-level recipe (no category)
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

    #[tokio::test]
    async fn test_create_with_default_path_root() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "---\ntitle: Simple Recipe\n---\n\nMix ingredients together.";
        let recipe = repo.create("Simple Recipe", content, None).await?;

        // Should be in root of recipes directory (no category)
        assert_eq!(recipe.category, None);
        assert!(recipe.git_path.starts_with("recipes/"));
        assert!(!recipe.git_path.contains("recipes/recipes"));
        assert_eq!(recipe.file_name, "simple-recipe.cook");

        // Verify it can be read back
        let read = repo.read(&recipe.git_path).await?;
        assert_eq!(read.name, "Simple Recipe");
        assert_eq!(read.category, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_with_hierarchical_path() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        let content = "---\ntitle: Chicken Biryani\n---\n\nBasmati rice with chicken.";
        let recipe = repo
            .create("Chicken Biryani", content, Some("meals/meat/traditional"))
            .await?;

        // Verify category and path structure
        assert_eq!(recipe.category, Some("meals/meat/traditional".to_string()));
        assert!(recipe.git_path.contains("meals/meat/traditional"));
        assert_eq!(recipe.file_name, "chicken-biryani.cook");

        // Verify category extraction from path
        let read = repo.read(&recipe.git_path).await?;
        assert_eq!(read.category, Some("meals/meat/traditional".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_missing_title_in_yaml_returns_error() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Content without YAML front matter title field
        let content = "---\ndescription: Some recipe\n---\n\nMix ingredients together.";
        let result = repo.create("Test", content, None).await;

        // Should return an error (invalid recipe content)
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("invalid recipe content"));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_generates_filename_from_title() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Test cases with various title formats
        let test_cases = vec![
            (
                "---\ntitle: Simple Cake\n---\n\ningredients",
                "simple-cake.cook",
            ),
            (
                "---\ntitle: Triple-Chocolate Cake!\n---\n\ningredients",
                "triple-chocolate-cake.cook",
            ),
            (
                "---\ntitle: Pad Thai (Thai Noodles)\n---\n\ningredients",
                "pad-thai-thai-noodles.cook",
            ),
        ];

        for (content, expected_filename) in test_cases {
            let recipe = repo
                .create("Test", content, Some("test"))
                .await?;
            assert_eq!(recipe.file_name, expected_filename);
            // Clean up for next iteration
            repo.delete(&recipe.git_path).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_only_content_with_title_change() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Create recipe with initial title
        let content = "---\ntitle: Chocolate Cake\n---\n\nBasic chocolate cake recipe.";
        let recipe = repo.create("Chocolate Cake", content, Some("desserts")).await?;

        let initial_git_path = recipe.git_path.clone();
        assert_eq!(recipe.file_name, "chocolate-cake.cook");

        // Update content with new title
        let new_content = "---\ntitle: Dark Chocolate Cake\n---\n\nRicher chocolate cake with dark cocoa.";
        let updated = repo
            .update(&initial_git_path, None, Some(new_content), None)
            .await?;

        // Verify git_path changed (file renamed)
        assert_ne!(updated.git_path, initial_git_path);
        assert_eq!(updated.name, "Dark Chocolate Cake");
        assert_eq!(updated.file_name, "dark-chocolate-cake.cook");
        assert!(updated.git_path.contains("dark-chocolate-cake"));

        // Verify old path no longer accessible
        assert!(repo.read(&initial_git_path).await.is_err());

        // Verify new path works
        let read = repo.read(&updated.git_path).await?;
        assert_eq!(read.name, "Dark Chocolate Cake");
        assert_eq!(read.content, new_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_only_path() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Create recipe in desserts category
        let content = "---\ntitle: Vanilla Cake\n---\n\nSimple vanilla cake.";
        let recipe = repo.create("Vanilla Cake", content, Some("desserts")).await?;

        let initial_git_path = recipe.git_path.clone();
        assert_eq!(recipe.category, Some("desserts".to_string()));

        // Update category only (path change)
        let updated = repo
            .update(&initial_git_path, None, None, Some(Some("mains")))
            .await?;

        // Verify git_path changed
        assert_ne!(updated.git_path, initial_git_path);
        assert_eq!(updated.category, Some("mains".to_string()));
        // Filename and title should stay the same
        assert_eq!(updated.name, "Vanilla Cake");
        assert_eq!(updated.file_name, "vanilla-cake.cook");

        // Verify old path no longer accessible
        assert!(repo.read(&initial_git_path).await.is_err());

        // Verify new path works
        let read = repo.read(&updated.git_path).await?;
        assert_eq!(read.name, "Vanilla Cake");
        assert_eq!(read.category, Some("mains".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_both_content_and_path() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Create recipe
        let content = "---\ntitle: Pasta\n---\n\nSimple pasta.";
        let recipe = repo.create("Pasta", content, Some("mains")).await?;

        let initial_git_path = recipe.git_path.clone();
        assert_eq!(recipe.category, Some("mains".to_string()));

        // Update both content (new title) AND category
        let new_content = "---\ntitle: Spaghetti Carbonara\n---\n\nClassic Italian pasta.";
        let updated = repo
            .update(&initial_git_path, None, Some(new_content), Some(Some("italian")))
            .await?;

        // Verify both changes
        assert_ne!(updated.git_path, initial_git_path);
        assert_eq!(updated.name, "Spaghetti Carbonara");
        assert_eq!(updated.file_name, "spaghetti-carbonara.cook");
        assert_eq!(updated.category, Some("italian".to_string()));
        assert!(updated.git_path.contains("italian"));
        assert!(updated.git_path.contains("spaghetti-carbonara"));

        // Verify old path no longer accessible
        assert!(repo.read(&initial_git_path).await.is_err());

        // Verify new path works
        let read = repo.read(&updated.git_path).await?;
        assert_eq!(read.name, "Spaghetti Carbonara");
        assert_eq!(read.content, new_content);
        assert_eq!(read.category, Some("italian".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_detects_file_misalignment() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Create recipe
        let content = "---\ntitle: Brownie\n---\n\nChocolate brownie.";
        let recipe = repo.create("Brownie", content, Some("desserts")).await?;

        let initial_git_path = recipe.git_path.clone();
        assert_eq!(recipe.file_name, "brownie.cook");

        // Imagine the file got misaligned (e.g., manually renamed in storage).
        // When we update the recipe without changing the title, it should still
        // detect that the filename doesn't match the generated name.
        // The update will trigger a rename if the generated filename differs.

        // Update content (same title) - this should not trigger rename
        let updated_content = "---\ntitle: Brownie\n---\n\nChocolate brownie recipe improved.";
        let updated = repo
            .update(&initial_git_path, None, Some(updated_content), None)
            .await?;

        // Same filename and path since title didn't change
        assert_eq!(updated.git_path, initial_git_path);
        assert_eq!(updated.file_name, "brownie.cook");

        // Now update with different title - should trigger rename
        let new_title_content = "---\ntitle: Fudgy Brownie\n---\n\nExtra fudgy brownie.";
        let renamed = repo
            .update(&updated.git_path, None, Some(new_title_content), None)
            .await?;

        assert_ne!(renamed.git_path, updated.git_path);
        assert_eq!(renamed.file_name, "fudgy-brownie.cook");

        Ok(())
    }

    #[tokio::test]
    async fn test_recipe_id_changes_on_rename() -> Result<()> {
        let (repo, _git) = setup_test_repo().await?;

        // Create recipe
        let content = "---\ntitle: Cake\n---\n\nSimple cake.";
        let recipe = repo.create("Cake", content, Some("desserts")).await?;

        let initial_git_path = recipe.git_path.clone();
        // Get the recipe_id from cache (simulating API response)
        let initial_recipe_id = repo
            .cache
            .get(&initial_git_path)
            .map(|c| c.recipe_id.clone())
            .expect("Recipe should be in cache");

        // Update with new title (causes rename)
        let new_content = "---\ntitle: Chocolate Cake\n---\n\nChocolate cake recipe.";
        let updated = repo
            .update(&initial_git_path, None, Some(new_content), None)
            .await?;

        // Get the new recipe_id
        let new_recipe_id = repo
            .cache
            .get(&updated.git_path)
            .map(|c| c.recipe_id.clone())
            .expect("Updated recipe should be in cache");

        // recipe_id should change because git_path changed
        assert_ne!(initial_recipe_id, new_recipe_id);

        // Old recipe_id should no longer be valid (old git_path no longer in cache)
        assert!(repo.cache.get(&initial_git_path).is_none());

        // New recipe_id should be retrievable
        assert_eq!(repo.get_recipe_git_path(&new_recipe_id), Some(updated.git_path.clone()));

        Ok(())
    }
}
