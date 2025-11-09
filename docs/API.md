# Cooklang Store API Documentation

## Overview

The Cooklang Store API provides RESTful endpoints for managing recipes stored in a git repository. All recipes are stored as `.cook` files and tracked in git for version history and collaboration.

## API Version

- **Current Version**: v1
- **Base URL**: `/api/v1`

## Common Response Format

### Success Response (2xx)
```json
{
  "recipe_id": "a1b2c3d4e5f6",
  "name": "Chocolate Cake",
  "description": null,
  "category": "desserts",
  "content": "# Recipe content..."
}
```

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
    "name": "Chocolate Cake",
    "content": "# Chocolate Cake\n\n@flour{2%cups}...",
    "category": "desserts",
    "author": "Alice",
    "comment": "Classic recipe from grandma"
  }
  ```
- **Response**:
  ```json
  {
    "recipe_id": "a1b2c3d4e5f6",
    "name": "Chocolate Cake",
    "description": null,
    "category": "desserts",
    "content": "# Chocolate Cake\n\n@flour{2%cups}..."
  }
  ```
- **Status Code**: `201 Created`
- **Validation**:
  - `name` is required and cannot be empty
  - `content` is required and cannot be empty
  - Must be valid Cooklang format

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
        "recipe_id": "a1b2c3d4e5f6",
        "name": "Chocolate Cake",
        "description": null,
        "category": "desserts"
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
- **Response**: Same as List Recipes
- **Status Code**: `200 OK`
- **Validation**:
  - `q` cannot be empty

#### Get Single Recipe
- **URL**: `/api/v1/recipes/{recipe_id}`
- **Method**: `GET`
- **Path Parameters**:
  - `recipe_id` (required): Unique recipe identifier (12-character hex string)
- **Response**:
  ```json
  {
    "recipe_id": "a1b2c3d4e5f6",
    "name": "Chocolate Cake",
    "description": null,
    "category": "desserts",
    "content": "# Chocolate Cake\n\n@flour{2%cups}..."
  }
  ```
- **Status Code**: `200 OK`
- **Error Codes**:
  - `404 Not Found`: Recipe not found

#### Update Recipe
- **URL**: `/api/v1/recipes/{recipe_id}`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **Path Parameters**:
  - `recipe_id` (required): Unique recipe identifier
- **Request Body** (all fields optional):
  ```json
  {
    "name": "New Name",
    "content": "Updated content...",
    "category": "new-category",
    "author": "Bob",
    "comment": "Updated ingredients"
  }
  ```
- **Response**: Full updated recipe
- **Status Code**: `200 OK`
- **Error Codes**:
  - `404 Not Found`: Recipe not found
  - `400 Bad Request`: Invalid content or validation failed

#### Delete Recipe
- **URL**: `/api/v1/recipes/{recipe_id}`
- **Method**: `DELETE`
- **Path Parameters**:
  - `recipe_id` (required): Unique recipe identifier
- **Response**: Empty body
- **Status Code**: `204 No Content`
- **Error Codes**:
  - `404 Not Found`: Recipe not found

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
  - `/api/v1/categories/desserts` - Get all recipes in the `desserts` category
  - `/api/v1/categories/meals%2Fmeat%2Ftraditional` - Get all recipes in `meals/meat/traditional`
- **Response**:
  ```json
  {
    "category": "desserts",
    "count": 12,
    "recipes": [
      {
        "recipe_id": "a1b2c3d4e5f6",
        "name": "Chocolate Cake",
        "description": null,
        "category": "desserts"
      }
    ]
  }
  ```
- **Status Code**: `200 OK`
- **Error Codes**:
  - `404 Not Found`: Category not found

## Categories

Categories reflect the directory structure on disk and support hierarchical nesting:

- **Single-level**: `recipes/desserts/chocolate-cake.cook` → category: `desserts`
- **Hierarchical**: `recipes/meals/meat/traditional/chicken-biryani.cook` → category: `meals/meat/traditional`
- **Root-level**: `recipes/simple-recipe.cook` → no category (optional field is null)

When creating or updating recipes, provide the category as a single string. If nesting is needed, use forward slashes (`/`) as directory separators.

## Recipe ID Format

Recipe IDs are deterministic 12-character hexadecimal strings generated from the SHA256 hash of the recipe's git path. This allows:
- Consistent IDs for the same recipe
- Client-side ID generation if the git path is known
- URL-friendly format (alphanumeric only)

Example: `a1b2c3d4e5f6`

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
    "name": "Pasta Carbonara",
    "content": "# Pasta Carbonara\n\n@eggs{4} @bacon{200%g} @pasta{400%g}",
    "category": "mains",
    "author": "Chef Alice"
  }'
```

### Search for Recipes
```bash
curl "http://localhost:3000/api/v1/recipes/search?q=chocolate&limit=10"
```

### Get a Specific Recipe
```bash
curl http://localhost:3000/api/v1/recipes/a1b2c3d4e5f6
```

### List Categories
```bash
curl http://localhost:3000/api/v1/categories
```

## Authentication

Currently, the API does not require authentication. This is planned for Phase 4 (User Management).

## Rate Limiting

Rate limiting is not currently implemented. This is planned for Phase 6 (Production Readiness).

## Versioning

The API follows semantic versioning:
- Current: `/api/v1`
- Future versions would be available at `/api/v2`, etc.

The version is also included in the status endpoint response.
