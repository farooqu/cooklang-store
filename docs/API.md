# Cooklang Store API Documentation

## Overview

The Cooklang Store API provides RESTful endpoints for managing recipes stored in a git repository. All recipes are stored as `.cook` files and tracked in git for version history and collaboration.

**Key Design**: Recipe names (titles) are derived from Cooklang YAML front matter metadata, not provided by the client. File names on disk are automatically generated from recipe titles and kept in sync.

## API Version

- **Current Version**: v1
- **Base URL**: `/api/v1`

## Common Response Format

### RecipeResponse (Full Recipe)
Used when retrieving individual recipes or after create/update operations.

```json
{
  "recipeId": "a1b2c3d4e5f6",
  "recipeName": "Chocolate Cake",
  "path": "desserts",
  "fileName": "chocolate-cake.cook",
  "description": null,
  "content": "---\ntitle: Chocolate Cake\n---\n\n# Recipe content..."
}
```

**Notes**:
- `recipeName` is derived from the `title` field in YAML front matter
- `fileName` is generated from the recipe name (lowercase, spaces→hyphens, `.cook` extension)
- `path` represents the directory location (relative to data-dir, no `recipes/` prefix)
- `description` is omitted from JSON if null (using `skip_serializing_if`)
- `content` always includes YAML front matter with title

### RecipeSummary (Compact Recipe)
Used in list and search endpoints.

```json
{
  "recipeId": "a1b2c3d4e5f6",
  "recipeName": "Chocolate Cake",
  "path": "desserts"
}
```

**Notes**:
- No `fileName` or `content` in summaries
- `path` omitted from JSON if null

### Error Response (4xx, 5xx)
```json
{
  "error": "error_code",
  "message": "Human-readable error message",
  "details": {
    "field": "Additional context"
  }
}
```

## Recipe Format: YAML Front Matter

All recipe content must include YAML front matter with a `title` field at the start:

```cook
---
title: Chocolate Cake
---

# Instructions

@flour{2%cups}
@sugar{1%cup}
```

**Format**:
- Delimited by `---` on its own lines (start and end)
- Must contain at least `title: Recipe Name`
- Can include additional metadata fields

**Validation**:
- Create and update operations validate that content includes YAML front matter with `title` field
- Missing title → 400 Bad Request

## Endpoints

### Health & Status

#### Health Check
- **URL**: `/health`
- **Method**: `GET`
- **Description**: Simple health check endpoint
- **Response**: `OK` (plain text)
- **Status Code**: `200 OK`

#### Status
- **URL**: `/api/v1/status`
- **Method**: `GET`
- **Description**: Get server status and recipe statistics
- **Response**:
  ```json
  {
    "status": "running",
    "version": "0.1.0",
    "recipe_count": 42,
    "categories": 8
  }
  ```

### Recipe CRUD Operations

#### Create Recipe
- **URL**: `/api/v1/recipes`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
  ```json
  {
    "content": "---\ntitle: Chocolate Cake\n---\n\n@flour{2%cups}...",
    "path": "desserts",
    "author": "Alice",
    "comment": "Classic recipe from grandma"
  }
  ```
  - `content` (required): Recipe in Cooklang format, must include YAML front matter with `title`
  - `path` (optional): Directory path for organization (defaults to root if omitted)
  - `author` (optional): Author name for git commit
  - `comment` (optional): Commit message
- **Response**:
  ```json
  {
    "recipeId": "a1b2c3d4e5f6",
    "recipeName": "Chocolate Cake",
    "path": "desserts",
    "fileName": "chocolate-cake.cook",
    "description": null,
    "content": "---\ntitle: Chocolate Cake\n---\n\n@flour{2%cups}..."
  }
  ```
- **Status Code**: `201 Created`
- **Validation**:
  - `content` is required and cannot be empty
  - `content` must include valid YAML front matter with `title` field
  - Missing title → 400 Bad Request

#### List Recipes
- **URL**: `/api/v1/recipes`
- **Method**: `GET`
- **Query Parameters**:
  - `limit` (optional): Items per page (default: 20, max: 100)
  - `offset` (optional): Items to skip (default: 0)
- **Response**:
  ```json
  {
    "recipes": [
      {
        "recipeId": "a1b2c3d4e5f6",
        "recipeName": "Chocolate Cake",
        "path": "desserts"
      }
    ],
    "pagination": {
      "limit": 20,
      "offset": 0,
      "total": 42
    }
  }
  ```
- **Status Code**: `200 OK`

#### Search Recipes
- **URL**: `/api/v1/recipes/search`
- **Method**: `GET`
- **Query Parameters**:
  - `q` (required): Search query (case-insensitive substring match on recipe name)
  - `limit` (optional): Items per page (default: 20, max: 100)
  - `offset` (optional): Items to skip (default: 0)
- **Response**: Same as List Recipes (array of RecipeSummary)
- **Status Code**: `200 OK`
- **Validation**:
  - `q` cannot be empty

#### Get Single Recipe
- **URL**: `/api/v1/recipes/{recipe_id}`
- **Method**: `GET`
- **Path Parameters**:
  - `recipe_id` (required): Unique recipe identifier (12-character hex string)
- **Response**: Full RecipeResponse with all fields and content
- **Status Code**: `200 OK`
- **Error Codes**:
  - `404 Not Found`: Recipe not found

#### Update Recipe
- **URL**: `/api/v1/recipes/{recipe_id}`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **Path Parameters**:
  - `recipe_id` (required): Unique recipe identifier
- **Request Body** (at least one field required):
  ```json
  {
    "content": "---\ntitle: Dark Chocolate Cake\n---\n\n...",
    "path": "desserts",
    "author": "Bob",
    "comment": "Updated ingredients"
  }
  ```
  - `content` (optional): New recipe content. If provided, must include YAML front matter with `title` field
  - `path` (optional): New directory path. If provided, recipe is moved to this location
  - `author` (optional): Author name for git commit
  - `comment` (optional): Commit message
- **Response**: Full updated RecipeResponse
- **Status Code**: `200 OK`
- **File Renaming**: If recipe content is updated and the title changes, the file on disk is automatically renamed to match the new recipe name
- **Error Codes**:
  - `404 Not Found`: Recipe not found
  - `400 Bad Request`: No fields provided, or content provided but missing YAML front matter with title

#### Delete Recipe
- **URL**: `/api/v1/recipes/{recipe_id}`
- **Method**: `DELETE`
- **Path Parameters**:
  - `recipe_id` (required): Unique recipe identifier
- **Response**: Empty body
- **Status Code**: `204 No Content`
- **Error Codes**:
  - `404 Not Found`: Recipe not found

### Fallback Lookup Endpoints

These endpoints help clients find recipes when recipe IDs change due to rename operations.

#### Find Recipes by Name
- **URL**: `/api/v1/recipes/find-by-name`
- **Method**: `GET`
- **Query Parameters**:
  - `q` (required): Recipe name search term (case-insensitive substring match)
  - `limit` (optional): Items per page (default: 20, max: 100)
  - `offset` (optional): Items to skip (default: 0)
- **Description**: Search for recipes by name. Use this when a recipe ID has changed due to a rename.
- **Response**: Array of RecipeSummary
  ```json
  {
    "recipes": [
      {
        "recipeId": "a1b2c3d4e5f6",
        "recipeName": "Chocolate Cake",
        "path": "desserts"
      }
    ],
    "pagination": {
      "limit": 20,
      "offset": 0,
      "total": 1
    }
  }
  ```
- **Status Code**: `200 OK` (returns empty array if no matches, not 404)

#### Find Recipe by Path
- **URL**: `/api/v1/recipes/find-by-path`
- **Method**: `GET`
- **Query Parameters**:
  - `path` (required): Recipe directory path (relative to data-dir, no `recipes/` prefix)
- **Description**: Find a recipe at a specific path. Use this when you know the location but not the recipe ID.
- **Response**: Single RecipeSummary (wrapped in object)
  ```json
  {
    "recipe": {
      "recipeId": "a1b2c3d4e5f6",
      "recipeName": "Chocolate Cake",
      "path": "desserts"
    }
  }
  ```
- **Status Code**: `200 OK`
- **Error Codes**:
  - `404 Not Found`: Recipe at that path not found

### Categories

#### List All Categories
- **URL**: `/api/v1/categories`
- **Method**: `GET`
- **Response**:
  ```json
  {
    "categories": ["appetizers", "desserts", "mains", "sides"]
  }
  ```
- **Status Code**: `200 OK`

#### Get Recipes in Category
- **URL**: `/api/v1/categories/{name}`
- **Method**: `GET`
- **Path Parameters**:
  - `name` (required): Category name (supports hierarchical paths with `/` separators)
- **Description**: Categories can be hierarchical, reflecting the directory structure. Use URL encoding for `/` as `%2F`.
- **Examples**:
  - `/api/v1/categories/desserts` - Get all recipes in the `desserts` directory
  - `/api/v1/categories/meals%2Fmeat%2Ftraditional` - Get all recipes in `meals/meat/traditional`
- **Response**:
  ```json
  {
    "category": "desserts",
    "count": 12,
    "recipes": [
      {
        "recipeId": "a1b2c3d4e5f6",
        "recipeName": "Chocolate Cake",
        "path": "desserts"
      }
    ]
  }
  ```
- **Status Code**: `200 OK`
- **Error Codes**:
  - `404 Not Found`: Category not found

## Recipe ID Stability

**Important**: Recipe IDs are derived from the recipe's file path (git_path) using a SHA256 hash. When a recipe is renamed (due to title change), its ID will change.

### Behavior
- IDs are **stable across content edits** (same file = same path = same ID)
- IDs **change on rename** (title change triggers automatic file rename on disk)
- IDs are deterministic (same path always produces same ID)

### Client Handling
If a bookmarked recipe ID returns 404:
1. Use `GET /api/v1/recipes/find-by-name?q=recipe-name` to search by name
2. Use `GET /api/v1/recipes/find-by-path?path=category/name` if you know the path
3. Clients should not rely on recipe IDs as permanent identifiers

## File Name Generation

File names are automatically generated from recipe titles using these rules:
- Convert to lowercase
- Replace spaces with hyphens
- Remove special characters (keep only alphanumeric and hyphens)
- Append `.cook` extension

Examples:
- "Chocolate Cake" → `chocolate-cake.cook`
- "Pasta Carbonara (Italian)" → `pasta-carbonara-italian.cook`
- "Sweet & Sour" → `sweet-sour.cook`

File names are kept synchronized with recipe titles. When you update a recipe's title, its file name is automatically updated on disk.

## Pagination

Pagination is supported on list and search endpoints:
- `limit`: Number of items to return (capped at 100, default 20)
- `offset`: Number of items to skip (default 0)
- `total`: Total number of items available (in response)

Example: `/api/v1/recipes?limit=10&offset=20`

## Error Handling

All errors return appropriate HTTP status codes:

- `200 OK`: Successful GET request
- `201 Created`: Successful POST (resource created)
- `204 No Content`: Successful DELETE
- `400 Bad Request`: Invalid input or validation failure
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

Error responses include:
- `error`: Machine-readable error code
- `message`: Human-readable error description
- `details` (optional): Additional context about the error

## Request/Response Encoding

- **Content-Type**: `application/json`
- **Character Encoding**: UTF-8
- **Max Body Size**: 10MB (for recipe content)

## CORS

The API has CORS enabled with permissive policy to allow requests from any origin.

## Examples

### Create a Recipe
```bash
curl -X POST http://localhost:3000/api/v1/recipes \
  -H "Content-Type: application/json" \
  -d '{
    "content": "---\ntitle: Pasta Carbonara\n---\n\n# Instructions\n\n@eggs{4} @bacon{200%g} @pasta{400%g}",
    "path": "mains",
    "author": "Chef Alice"
  }'
```

Response (201 Created):
```json
{
  "recipeId": "a1b2c3d4e5f6",
  "recipeName": "Pasta Carbonara",
  "path": "mains",
  "fileName": "pasta-carbonara.cook",
  "content": "---\ntitle: Pasta Carbonara\n---\n\n# Instructions\n\n@eggs{4} @bacon{200%g} @pasta{400%g}"
}
```

### Search for Recipes
```bash
curl "http://localhost:3000/api/v1/recipes/search?q=chocolate&limit=10"
```

Response (200 OK):
```json
{
  "recipes": [
    {
      "recipeId": "a1b2c3d4e5f6",
      "recipeName": "Chocolate Cake",
      "path": "desserts"
    }
  ],
  "pagination": {
    "limit": 10,
    "offset": 0,
    "total": 1
  }
}
```

### Get a Specific Recipe
```bash
curl http://localhost:3000/api/v1/recipes/a1b2c3d4e5f6
```

### Update Recipe (Change Title)
```bash
curl -X PUT http://localhost:3000/api/v1/recipes/a1b2c3d4e5f6 \
  -H "Content-Type: application/json" \
  -d '{
    "content": "---\ntitle: Dark Chocolate Cake\n---\n\n# New instructions..."
  }'
```

Note: Recipe file on disk will be renamed from `chocolate-cake.cook` to `dark-chocolate-cake.cook`.

### Find Recipe by Name (After Rename)
If the recipe ID has changed due to a rename:
```bash
curl "http://localhost:3000/api/v1/recipes/find-by-name?q=Dark%20Chocolate"
```

### Find Recipe by Path
```bash
curl "http://localhost:3000/api/v1/recipes/find-by-path?path=desserts"
```

### List Categories
```bash
curl http://localhost:3000/api/v1/categories
```

### Get Recipes in a Category
```bash
curl http://localhost:3000/api/v1/categories/mains
```

## Authentication

Currently, the API does not require authentication. This is planned for a future phase.

## Rate Limiting

Rate limiting is not currently implemented. This is planned for a future phase.

## Versioning

The API follows semantic versioning:
- Current: `/api/v1`
- Future versions would be available at `/api/v2`, etc.

The version is also included in the status endpoint response.
