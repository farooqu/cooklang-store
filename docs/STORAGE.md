# Storage Architecture

## Overview

The Cooklang Store supports two storage modes for recipe files:

1. **Disk Storage** (Default) - Simple, fast, no version history
2. **Git Storage** (Optional) - Version-tracked, collaboration-friendly

Both modes store recipes as `.cook` files on disk with identical content structure. The difference is in version history tracking.

## Storage Modes

### Disk Storage (Default)

**When to use**: Single-user, family-scale deployments without need for version history or branching.

- **Mechanism**: Direct filesystem operations (read/write/delete)
- **Version History**: None - only current file state is kept
- **Commit Tracking**: Not available
- **Use Cases**:
  - Personal recipe collection
  - Family shared recipes (no collaboration audit needed)
  - Lightweight deployments
  - Simplest operational model

**Configuration**:
```bash
STORAGE_TYPE=disk  # or omit, it's the default
```

### Git Storage (Optional)

**When to use**: Deployments that need recipe change history, audit trails, or multi-user contributions with version control.

- **Mechanism**: Git repository with automatic commits on all changes
- **Version History**: Full git history with author, timestamp, and commit messages
- **Commit Tracking**: Every create/update/delete operation creates a git commit
- **Use Cases**:
  - Team recipe management
  - Audit trails for recipe changes
  - Rollback to previous versions
  - Collaborative recipe development

**Configuration**:
```bash
STORAGE_TYPE=git
```

## File Structure

Both modes organize recipes the same way:

```
recipes/
├── simple-recipe.cook           # Root level recipe (no category)
├── desserts/
│   ├── chocolate-cake.cook      # category: "desserts"
│   └── vanilla-cake.cook
├── mains/
│   └── pasta.cook               # category: "mains"
└── meals/
    └── meat/
        └── traditional/
            └── chicken-biryani.cook  # category: "meals/meat/traditional"
```

The file paths are used to:
- Generate recipe IDs (SHA256 hash of git_path)
- Extract hierarchical categories (all directory levels between `recipes/` and the filename)
- Create URL-friendly slugs from recipe names

## Implementation Details

### Storage Trait

Both backends implement the `RecipeStorage` trait:

```rust
pub trait RecipeStorage: Send + Sync {
    async fn write_file(&self, rel_path: &str, content: &str) -> Result<()>;
    async fn read_file(&self, rel_path: &str) -> Result<String>;
    async fn delete_file(&self, rel_path: &str) -> Result<()>;
    async fn discover_files(&self) -> Result<Vec<String>>;
}
```

### Initialization

The storage backend is selected at startup based on `STORAGE_TYPE` environment variable or `--storage` CLI argument:

```rust
let storage = match storage_type {
    "git" => Box::new(GitStorage::new(repo_path).await?),
    "disk" => Box::new(DiskStorage::new(repo_path).await?),
    _ => Box::new(DiskStorage::new(repo_path).await?), // default
};
```

## Migration Between Modes

### From Disk to Git

To add version history to an existing disk-based deployment:

1. Initialize a git repository in the recipes directory
2. Commit all existing files
3. Switch `STORAGE_TYPE=git` or pass `--storage git` on startup
4. Restart the server

All subsequent operations will create git commits.

### From Git to Disk

To remove git overhead from a git-backed deployment:

1. Switch `STORAGE_TYPE=disk` or pass `--storage disk` on startup
2. Restart the server
3. Optionally, clean up `.git` directory (though it won't be used)

Existing recipe files remain intact and operational.

## Caching Strategy

Both storage modes use the same in-memory cache (`DashMap`) for fast access:

- Cache is built at startup by discovering all `.cook` files
- Cache keys are git paths (e.g., `recipes/desserts/cake.cook`)
- Cache stores parsed recipes, names, categories, and recipe IDs
- Cache is only consulted for lookups; writes always go to storage first
- Cache is rebuilt on each operation to ensure consistency

## Performance Characteristics

| Operation | Disk | Git |
|-----------|------|-----|
| Create | Fast (1 write) | Slower (write + git commit) |
| Read | Fast (cache + file read) | Fast (cache + file read) |
| Update | Fast (1 write) | Slower (write + git commit) |
| Delete | Fast (1 delete) | Slower (delete + git commit) |
| List/Search | Fast (from cache) | Fast (from cache) |

Git mode adds commit overhead (~50-100ms per write operation depending on repository size).

## Security & Safety

### Both Modes
- File paths are validated to prevent directory traversal
- `.cook` files are restricted to the recipes directory
- Invalid file content (malformed recipes) is rejected during parsing

### Git Mode Additional
- Git operations are atomic (commit succeeds or fails as a unit)
- Commit history provides audit trail of changes
- Can revert changes by reverting commits

### Disk Mode Considerations
- No atomic operations - partial writes could occur if system crashes
- No version history - deleted recipes cannot be recovered
- Suitable for simple deployments where data loss is acceptable

For critical deployments, use Git mode or implement external backups.

## Configuration Examples

### Docker Compose - Disk (Default)

```yaml
environment:
  - STORAGE_TYPE=disk  # optional, disk is default
```

### Docker Compose - Git

```yaml
environment:
  - STORAGE_TYPE=git
```

### Local Development

```bash
# Disk mode (with environment variable)
export STORAGE_TYPE=disk
cargo run -- --data-dir ./recipes

# Git mode (with environment variable)
export STORAGE_TYPE=git
cargo run -- --data-dir ./recipes

# Or with CLI arguments directly (overrides env vars)
cargo run -- --data-dir ./recipes --storage disk
cargo run -- --data-dir ./recipes --storage git
```

## Category Structure

Categories support hierarchical nesting to reflect the directory structure on disk:

- `recipes/desserts/cake.cook` → category: `desserts`
- `recipes/meals/meat/traditional/chicken-biryani.cook` → category: `meals/meat/traditional`
- `recipes/simple.cook` → no category (root level)

The category field contains the full path from `recipes/` to the parent directory of the file, with directory separators preserved as forward slashes `/`.

## Compatibility

- **No breaking changes** - API responses are identical regardless of storage mode
- **Recipe IDs** are consistent across modes (based on file paths)
- **Category extraction** works the same way and supports hierarchical paths
- **All CRUD operations** work identically from the API perspective

The storage backend is purely an implementation detail invisible to API consumers.
