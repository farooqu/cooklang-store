use anyhow::Result;
use std::path::Path;

pub mod disk;
pub mod git;

pub use disk::DiskStorage;
pub use git::GitStorage;

/// Trait for recipe file storage backends
pub trait RecipeStorage: Send + Sync {
    /// Write a file to storage
    fn write_file(&self, rel_path: &str, content: &str) -> Result<()>;

    /// Read a file from storage
    fn read_file(&self, rel_path: &str) -> Result<String>;

    /// Delete a file from storage
    fn delete_file(&self, rel_path: &str) -> Result<()>;

    /// Discover all .cook files in storage
    fn discover_files(&self) -> Result<Vec<String>>;
}

/// Create a storage backend based on configuration
pub async fn create_storage(
    storage_type: &str,
    repo_path: &Path,
) -> Result<Box<dyn RecipeStorage>> {
    match storage_type {
        "git" => Ok(Box::new(GitStorage::new(repo_path)?)),
        _ => Ok(Box::new(DiskStorage::new(repo_path)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_disk_storage() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = create_storage("disk", temp_dir.path()).await?;

        // Should create storage without error
        assert!(storage.discover_files()?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_disk_storage_default() -> Result<()> {
        let temp_dir = TempDir::new()?;
        // Omitting "disk" explicitly, using empty string which defaults to disk
        let storage = create_storage("unknown", temp_dir.path()).await?;

        assert!(storage.discover_files()?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_git_storage() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = create_storage("git", temp_dir.path()).await?;

        assert!(storage.discover_files()?.is_empty());

        Ok(())
    }
}
