# Phase 5: Documentation & Examples

**Status**: ❌ NOT STARTED
**Milestone**: category-path-refactor
**Phase**: 5 (Documentation & Examples)
**Branch**: feat/category-path-refactor

---

## Task 5.1: Update Postman Collection

- [ ] Add/update Create Recipe request with new payload format (content + path + author + comment)
- [ ] Add/update Update Recipe request with new payload format (optional content and/or path)
- [ ] Add new requests for fallback endpoints:
   - [ ] `GET /recipes/find-by-name?q=Chocolate`
   - [ ] `GET /recipes/find-by-path?path=desserts`
- [ ] Update all response examples to show new RecipeResponse and RecipeSummary schemas
- [ ] Add examples for path parameter (empty for root, hierarchical paths like `desserts/chocolate`)
- [ ] Add example showing ID change scenario: create recipe → rename via title → lookup by name
- [ ] Validate JSON syntax: `python3 -m json.tool docs/postman-collection.json > /dev/null`

## Task 5.2: Update Sample Recipes

- [ ] Ensure all sample recipes have proper Cooklang metadata with titles
- [ ] Update curl examples to match new request format
- [ ] Add example showing file renaming behavior
- [ ] Test all examples work with actual API

## Task 5.3: Update README if Needed

- [ ] Verify quick start examples work with new API
- [ ] Update any architecture notes about recipe naming

---

## Definition of Done

- [x] Postman collection updated with all new endpoints and request/response formats
- [x] Postman collection validates successfully
- [x] Sample recipes all have YAML front matter with titles
- [x] curl examples in documentation match new API
- [x] README quick start examples work and are up-to-date
- [x] All documentation examples tested manually
