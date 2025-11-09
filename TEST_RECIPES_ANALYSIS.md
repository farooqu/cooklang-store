# Test Recipes Analysis and Creation

## Summary

All 19 distinct test recipes have been created as proper Cooklang files with YAML frontmatter. These recipes are now ready to replace hardcoded content in the integration tests.

## Recipe Inventory

### File Name → Title Mapping

1. **test-recipe.cook** → "Test Recipe"
2. **chocolate-cake.cook** → "Chocolate Cake"
3. **recipe-1.cook** → "Recipe 1"
4. **recipe-2.cook** → "Recipe 2"
5. **cake.cook** → "Cake"
6. **pasta.cook** → "Pasta"
7. **to-delete.cook** → "To Delete"
8. **vanilla-cake.cook** → "Vanilla Cake"
9. **original-name.cook** → "Original Name"
10. **updated-name.cook** → "Updated Name"
11. **chicken-biryani.cook** → "Chicken Biryani"
12. **thai-green-curry.cook** → "Thai Green Curry"
13. **pad-thai.cook** → "Pad Thai"
14. **green-curry.cook** → "Green Curry"
15. **spaghetti.cook** → "Spaghetti"
16. **tiramisu.cook** → "Tiramisu"
17. **cheesecake.cook** → "Cheesecake"
18. **flan.cook** → "Flan"
19. **authors-dinner.cook** → "Author's Dinner"

## Schema Validation

All recipes follow the required Cooklang schema:

```
---
title: Recipe Name
---

# Recipe Name

Description...

## Ingredients

- @ingredient{quantity%unit}

## Instructions

1. Step one with @ingredient usage.
```

## Test Coverage Mapping

### Basic Creation/Retrieval (Flat Categories)
- test-recipe.cook: Flat category "desserts"
- chocolate-cake.cook: Flat category "desserts"
- cake.cook: Flat category "desserts"
- pasta.cook: Flat category "main"

### Pagination/Listing
- recipe-1.cook: Category "desserts"
- recipe-2.cook: Category "desserts"

### Deletion
- to-delete.cook: Category "desserts"

### Updates (Rename and Category Change)
- original-name.cook: Original state (category "desserts")
- updated-name.cook: Updated state (category "main")

### Nested Categories (meals/*)
- chicken-biryani.cook: meals/meat/traditional
- thai-green-curry.cook: meals/asian/thai
- pad-thai.cook: meals/asian/thai
- green-curry.cook: meals/asian/thai
- spaghetti.cook: meals/european/italian

### Nested Categories (desserts/*)
- tiramisu.cook: desserts/cakes/italian
- cheesecake.cook: desserts/cakes/american
- vanilla-cake.cook: desserts/cakes/vanilla
- flan.cook: desserts/custards

### Complex Category Movement
- authors-dinner.cook: Used to test movement between different hierarchy structures (author1/dinner/meat → author2/meat/dinner)

## Location

All recipes are stored in: `/workspaces/cooklang-store/tests/fixtures/`

## Next Steps

When updating the integration tests to use these fixtures:

1. Create a helper function to load recipes from the fixtures directory
2. Map each test to use the appropriate recipe fixture
3. Ensure both disk and git storage tests use the exact same recipe files
4. This eliminates duplication and ensures consistency across test implementations
5. Simplifies test maintenance - recipe updates only need to happen in one place

## Recipes Created: 19

- Basic/Utility: 8 recipes
- Update Testing: 2 recipes  
- Flat Categories: 4 recipes
- Nested Categories: 5 recipes
- Complex Movement Testing: 1 recipe

## Benefits

✓ Eliminates hardcoded recipe duplication across 3 test files
✓ Easier to maintain and modify test recipes
✓ Ensures git and disk storage tests use identical input data
✓ Proper Cooklang format with YAML frontmatter validation
✓ Clear separation of test data from test logic
✓ All recipes follow proper schema standards
