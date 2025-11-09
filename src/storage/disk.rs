use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::RecipeStorage;

/// Disk-based storage backend - stores recipes directly on filesystem without version control
pub struct DiskStorage {
    repo_path: PathBuf,
}

impl DiskStorage {
    /// Create a new disk storage instance
    pub fn new(repo_path: &Path) -> Result<Self> {
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(repo_path).context("Failed to create storage directory")?;

        Ok(DiskStorage {
            repo_path: repo_path.to_path_buf(),
        })
    }
}

impl RecipeStorage for DiskStorage {
    fn write_file(&self, rel_path: &str, content: &str) -> Result<()> {
        let full_path = self.repo_path.join(rel_path);

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create recipe directory")?;
        }

        std::fs::write(&full_path, content)
            .context(format!("Failed to write file: {}", rel_path))
    }

    fn read_file(&self, rel_path: &str) -> Result<String> {
        let full_path = self.repo_path.join(rel_path);

        std::fs::read_to_string(&full_path)
            .context(format!("Failed to read file: {}", rel_path))
    }

    fn delete_file(&self, rel_path: &str) -> Result<()> {
        let full_path = self.repo_path.join(rel_path);

        if full_path.exists() {
            std::fs::remove_file(&full_path)
                .context(format!("Failed to delete file: {}", rel_path))?;
        }

        Ok(())
    }

    fn discover_files(&self) -> Result<Vec<String>> {
        let mut cook_files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("cook") {
                let relative_path = entry
                    .path()
                    .strip_prefix(&self.repo_path)?
                    .to_string_lossy()
                    .to_string();
                cook_files.push(relative_path);
            }
        }

        Ok(cook_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().join("recipes");

        assert!(!path.exists());
        let _storage = DiskStorage::new(&path)?;
        assert!(path.exists());

        Ok(())
    }

    #[test]
    fn test_new_opens_existing_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().join("recipes");

        let _storage1 = DiskStorage::new(&path)?;
        let _storage2 = DiskStorage::new(&path)?; // Should not fail

        Ok(())
    }

    #[test]
    fn test_write_and_read_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = DiskStorage::new(temp_dir.path())?;

        let content = "# Test Recipe\n\n@ingredient{}";
        storage.write_file("recipes/test.cook", content)?;

        let read_content = storage.read_file("recipes/test.cook")?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_creates_parent_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = DiskStorage::new(temp_dir.path())?;

        storage.write_file("recipes/desserts/chocolate/cake.cook", "# Cake")?;

        let path = temp_dir.path().join("recipes/desserts/chocolate/cake.cook");
        assert!(path.exists());

        Ok(())
    }

    #[test]
    fn test_delete_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = DiskStorage::new(temp_dir.path())?;

        storage.write_file("recipes/test.cook", "# Test")?;
        assert!(temp_dir.path().join("recipes/test.cook").exists());

        storage.delete_file("recipes/test.cook")?;
        assert!(!temp_dir.path().join("recipes/test.cook").exists());

        Ok(())
    }

    #[test]
    fn test_delete_nonexistent_file_succeeds() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = DiskStorage::new(temp_dir.path())?;

        // Should not fail if file doesn't exist
        storage.delete_file("recipes/nonexistent.cook")?;

        Ok(())
    }

    #[test]
    fn test_discover_cook_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = DiskStorage::new(temp_dir.path())?;

        storage.write_file("recipes/test1.cook", "# Test 1")?;
        storage.write_file("recipes/desserts/cake.cook", "# Cake")?;
        storage.write_file("recipes/mains/pasta.cook", "# Pasta")?;
        storage.write_file("recipes/readme.txt", "Not a recipe")?; // Should be ignored

        let files = storage.discover_files()?;
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.contains("test1.cook")));
        assert!(files.iter().any(|f| f.contains("cake.cook")));
        assert!(files.iter().any(|f| f.contains("pasta.cook")));

        Ok(())
    }

    #[test]
    fn test_discover_empty_repository() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = DiskStorage::new(temp_dir.path())?;

        let files = storage.discover_files()?;
        assert_eq!(files.len(), 0);

        Ok(())
    }
}
