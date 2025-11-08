use anyhow::{Context, Result};
use git2::{Repository, Signature};
use std::path::Path;

/// Initializes a git repository at the given path
pub fn init_repo(path: &Path) -> Result<Repository> {
    if path.join(".git").exists() {
        // Repository already exists, open it
        Repository::open(path).context("Failed to open existing git repository")
    } else {
        // Create the directory and initialize repo
        std::fs::create_dir_all(path).context("Failed to create recipes directory")?;
        Repository::init(path).context("Failed to initialize git repository")
    }
}

/// Get or create the default git signature for commits
fn get_default_signature() -> Result<Signature<'static>> {
    Signature::now("CookLang Backend", "backend@cooklang.local")
        .context("Failed to create git signature")
}

/// Create a git signature with a specific author name
fn get_signature_with_author(author: &str) -> Result<Signature<'_>> {
    Signature::now(author, "backend@cooklang.local").context(format!(
        "Failed to create git signature for author: {}",
        author
    ))
}

/// Commit a single file to the repository
pub fn commit_file(repo: &Repository, rel_path: &str, message: &str) -> Result<git2::Oid> {
    commit_file_with_author(repo, rel_path, message, None)
}

/// Commit a single file with an optional author
pub fn commit_file_with_author(
    repo: &Repository,
    rel_path: &str,
    message: &str,
    author: Option<&str>,
) -> Result<git2::Oid> {
    let mut index = repo.index()?;
    index.add_path(Path::new(rel_path))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let signature = if let Some(author_name) = author {
        get_signature_with_author(author_name)?
    } else {
        get_default_signature()?
    };

    let parent_commit = match repo.head() {
        Ok(head) => {
            let commit = head.peel_to_commit()?;
            vec![commit]
        }
        Err(_) => {
            // First commit, no parent
            vec![]
        }
    };

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )?;

    Ok(oid)
}

/// Delete a file and commit the deletion
pub fn delete_file(repo: &Repository, rel_path: &str, message: &str) -> Result<git2::Oid> {
    delete_file_with_author(repo, rel_path, message, None)
}

/// Delete a file and commit the deletion with an optional author
pub fn delete_file_with_author(
    repo: &Repository,
    rel_path: &str,
    message: &str,
    author: Option<&str>,
) -> Result<git2::Oid> {
    let file_path = repo
        .workdir()
        .context("Repository has no working directory")?
        .join(rel_path);

    if file_path.exists() {
        std::fs::remove_file(&file_path).context("Failed to delete file from filesystem")?;
    }

    let mut index = repo.index()?;
    index.remove_path(Path::new(rel_path))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let signature = if let Some(author_name) = author {
        get_signature_with_author(author_name)?
    } else {
        get_default_signature()?
    };

    let parent_commit = repo.head()?.peel_to_commit()?;

    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(oid)
}

/// Read a file from the repository
pub fn read_file(repo: &Repository, rel_path: &str) -> Result<String> {
    let file_path = repo
        .workdir()
        .context("Repository has no working directory")?
        .join(rel_path);

    std::fs::read_to_string(&file_path).context(format!("Failed to read file: {}", rel_path))
}

/// Discover all .cook files in the repository recursively
pub fn discover_cook_files(repo: &Repository) -> Result<Vec<String>> {
    let workdir = repo
        .workdir()
        .context("Repository has no working directory")?;

    let mut cook_files = Vec::new();

    for entry in walkdir::WalkDir::new(workdir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("cook") {
            let relative_path = entry
                .path()
                .strip_prefix(workdir)?
                .to_string_lossy()
                .to_string();
            cook_files.push(relative_path);
        }
    }

    Ok(cook_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_repo_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");

        assert!(!repo_path.exists());
        let _repo = init_repo(&repo_path)?;
        assert!(repo_path.exists());
        assert!(repo_path.join(".git").exists());

        Ok(())
    }

    #[test]
    fn test_init_repo_opens_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");

        let _repo1 = init_repo(&repo_path)?;
        let _repo2 = init_repo(&repo_path)?; // Should open existing, not fail

        Ok(())
    }

    #[test]
    fn test_commit_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");
        let repo = init_repo(&repo_path)?;

        // Write a test file
        let test_file = repo_path.join("test.cook");
        std::fs::write(&test_file, "# Test Recipe")?;

        // Commit it
        let oid = commit_file(&repo, "test.cook", "Add test recipe")?;
        assert!(!oid.is_zero());

        Ok(())
    }

    #[test]
    fn test_commit_file_with_author() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");
        let repo = init_repo(&repo_path)?;

        // Write a test file
        let test_file = repo_path.join("test.cook");
        std::fs::write(&test_file, "# Test Recipe")?;

        // Commit with author
        let oid = commit_file_with_author(&repo, "test.cook", "Add test recipe", Some("Alice"))?;
        assert!(!oid.is_zero());

        // Verify the commit has the correct author
        let commit = repo.find_commit(oid)?;
        let author = commit.author();
        assert_eq!(author.name(), Some("Alice"));

        Ok(())
    }

    #[test]
    fn test_delete_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");
        let repo = init_repo(&repo_path)?;

        // Write and commit a test file
        let test_file = repo_path.join("test.cook");
        std::fs::write(&test_file, "# Test Recipe")?;
        commit_file(&repo, "test.cook", "Add test recipe")?;

        // Delete it
        let oid = delete_file(&repo, "test.cook", "Delete test recipe")?;
        assert!(!oid.is_zero());
        assert!(!test_file.exists());

        Ok(())
    }

    #[test]
    fn test_delete_file_with_author() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");
        let repo = init_repo(&repo_path)?;

        // Write and commit a test file
        let test_file = repo_path.join("test.cook");
        std::fs::write(&test_file, "# Test Recipe")?;
        commit_file(&repo, "test.cook", "Add test recipe")?;

        // Delete with author
        let oid = delete_file_with_author(&repo, "test.cook", "Delete test recipe", Some("Bob"))?;
        assert!(!oid.is_zero());

        // Verify the commit has the correct author
        let commit = repo.find_commit(oid)?;
        let author = commit.author();
        assert_eq!(author.name(), Some("Bob"));

        Ok(())
    }

    #[test]
    fn test_read_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("recipes");
        let repo = init_repo(&repo_path)?;

        let content = "# Test Recipe\n@ingredient{}";
        std::fs::write(repo_path.join("test.cook"), content)?;

        let read_content = read_file(&repo, "test.cook")?;
        assert_eq!(read_content, content);

        Ok(())
    }
}
