use anyhow::{anyhow, Context, Result};
use git2::Repository as GitRepository;
use std::path::Path;
use std::sync::Mutex;

use super::RecipeStorage;
use crate::git;

/// Git-based storage backend - maintains version history with automatic commits
pub struct GitStorage {
    repo: Mutex<GitRepository>,
}

impl GitStorage {
    /// Create a new git storage instance
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = git::init_repo(repo_path)?;

        Ok(GitStorage {
            repo: Mutex::new(repo),
        })
    }
}

impl RecipeStorage for GitStorage {
    fn write_file(&self, rel_path: &str, content: &str) -> Result<()> {
        let repo = self
            .repo
            .lock()
            .map_err(|_| anyhow!("Failed to lock git repository"))?;

        let workdir = repo
            .workdir()
            .context("Repository has no working directory")?;
        let full_path = workdir.join(rel_path);

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create recipe directory")?;
        }

        // Write the file
        std::fs::write(&full_path, content).context("Failed to write recipe file")?;

        // Commit the change
        let commit_message = format!("Update recipe: {}", rel_path);
        git::commit_file(&repo, rel_path, &commit_message)?;

        Ok(())
    }

    fn read_file(&self, rel_path: &str) -> Result<String> {
        let repo = self
            .repo
            .lock()
            .map_err(|_| anyhow!("Failed to lock git repository"))?;
        git::read_file(&repo, rel_path)
    }

    fn delete_file(&self, rel_path: &str) -> Result<()> {
        let repo = self
            .repo
            .lock()
            .map_err(|_| anyhow!("Failed to lock git repository"))?;

        let commit_message = format!("Delete recipe: {}", rel_path);
        git::delete_file(&repo, rel_path, &commit_message)?;

        Ok(())
    }

    fn discover_files(&self) -> Result<Vec<String>> {
        let repo = self
            .repo
            .lock()
            .map_err(|_| anyhow!("Failed to lock git repository"))?;
        git::discover_cook_files(&repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_creates_git_repo() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().join("recipes");

        assert!(!path.exists());
        let _storage = GitStorage::new(&path)?;
        assert!(path.join(".git").exists());

        Ok(())
    }

    #[test]
    fn test_write_file_commits() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = GitStorage::new(temp_dir.path())?;

        let content = "# Test Recipe\n\n@ingredient{}";
        storage.write_file("recipes/test.cook", content)?;

        // Verify file exists
        assert!(temp_dir.path().join("recipes/test.cook").exists());

        // Verify it's in git history
        let repo = storage.repo.lock().unwrap();
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        assert!(commit.message().unwrap().contains("Update recipe"));

        Ok(())
    }

    #[test]
    fn test_read_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = GitStorage::new(temp_dir.path())?;

        let content = "# Test Recipe\n\n@ingredient{}";
        storage.write_file("recipes/test.cook", content)?;

        let read_content = storage.read_file("recipes/test.cook")?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_delete_file_commits() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = GitStorage::new(temp_dir.path())?;

        // Create and then delete
        storage.write_file("recipes/test.cook", "# Test")?;
        assert!(temp_dir.path().join("recipes/test.cook").exists());

        storage.delete_file("recipes/test.cook")?;
        assert!(!temp_dir.path().join("recipes/test.cook").exists());

        // Verify deletion is committed
        let repo = storage.repo.lock().unwrap();
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        assert!(commit.message().unwrap().contains("Delete recipe"));

        Ok(())
    }

    #[test]
    fn test_discover_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = GitStorage::new(temp_dir.path())?;

        storage.write_file("recipes/test1.cook", "# Test 1")?;
        storage.write_file("recipes/desserts/cake.cook", "# Cake")?;
        storage.write_file("recipes/mains/pasta.cook", "# Pasta")?;

        let files = storage.discover_files()?;
        assert_eq!(files.len(), 3);

        Ok(())
    }
}
