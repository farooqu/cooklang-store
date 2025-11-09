# Task 5.1: Update Postman Collection

**Status**: ✅ COMPLETE
**Milestone**: category-path-refactor
**Phase**: 5 (Documentation & Examples)
**Branch**: feat/category-path-refactor

---

## Subtasks

- [x] Review current postman-collection.json structure
- [x] Update Create Recipe request example with response (201 Created)
  - [x] Set request body to new format: `{ "content": "...", "path": "desserts" }`
  - [x] Ensure content includes YAML front matter with `title` field
  - [x] Add author and comment as optional fields in example
- [x] Update Update Recipe requests:
  - [x] Renamed to "Update Recipe - Change Content (with Title Change)" to clarify ID change behavior
  - [x] Added response example showing fileName change and recipeId change
  - [x] Kept "Update Recipe - Change Path" with response showing path change
- [x] Find by Name (Fallback Lookup) endpoint:
  - [x] Updated description to clarify usage when recipe ID changes
  - [x] Added 200 OK response example with RecipeSummary array
- [x] Find by Path (Fallback Lookup) endpoint:
  - [x] Updated description to clarify usage
  - [x] Added 200 OK response example with single RecipeSummary
- [x] List Recipes response example with RecipeSummary format (recipeId, recipeName, path)
- [x] Get Recipe response example with full RecipeResponse format (recipeId, recipeName, path, fileName, content)
- [x] ID change scenario demonstrated:
  - [x] Create request + response showing initial recipe with ID a1b2c3d4e5f6
  - [x] Update request + response showing title change triggering fileName change and new ID b2c3d4e5f6a1
  - [x] Find-by-name response showing how to locate recipe after ID change
- [x] Delete Recipe response example (204 No Content)
- [x] Validate JSON syntax: `python3 -m json.tool docs/postman-collection.json > /dev/null`
- [x] Commit changes with message: "[category-path-refactor] Task 5.1 - Update Postman collection"

---

## Notes

- Postman collection file location: `docs/postman-collection.json`
- All request/response bodies should use camelCase field names
- RecipeSummary format: `{ "recipeId", "recipeName", "path" }`
- RecipeResponse format: `{ "recipeId", "recipeName", "path", "fileName", "content" }`
