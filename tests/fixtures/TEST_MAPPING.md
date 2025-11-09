# Test Recipe Fixture Mapping

This document shows how each recipe fixture is used across the integration tests.

## Usage by Test Type

### Health & Status Tests
- No recipe fixtures needed (test empty repository)

### Recipe Creation Tests
- **test_create_recipe()** → `test-recipe.cook`
- **test_create_recipe_with_comment()** → `chocolate-cake.cook`
- **test_create_recipe_empty_category()** → `test-recipe.cook`

### Recipe Retrieval Tests
- **test_get_recipe_by_id()** → `test-recipe.cook`
- **test_list_recipes_with_pagination()** → `recipe-1.cook`, `recipe-2.cook`
- **test_list_recipes_with_limit()** → `recipe-1.cook`, `recipe-2.cook`, `test-recipe.cook`

### Recipe Search Tests
- **test_search_recipes_by_name()** → `chocolate-cake.cook`, `vanilla-cake.cook`, `pasta.cook`
- **test_search_case_insensitive()** → `chocolate-cake.cook`

### Category Tests
- **test_list_categories()** → `cake.cook`, `pasta.cook`, `test-recipe.cook`
- **test_get_recipes_in_category()** → `cake.cook`, `pasta.cook`

### Recipe Update Tests
- **test_update_recipe()** → `original-name.cook` (create), then update to use `updated-name.cook` content

### Recipe Delete Tests
- **test_delete_recipe()** → `to-delete.cook`

### Status After Modifications
- **test_status_updates_with_recipes()** → `cake.cook`, `pasta.cook`

### Nested Category Tests (meals/*)
- **test_create_recipe_in_nested_category()** → `chicken-biryani.cook`
- **test_read_recipe_from_nested_category()** → `thai-green-curry.cook`
- **test_get_recipes_from_nested_category()** → `pad-thai.cook`, `green-curry.cook`, `spaghetti.cook`
- **test_list_categories_includes_nested()** → `tiramisu.cook`, `cheesecake.cook`, `flan.cook`
- **test_move_recipe_between_nested_categories()** → `chocolate-cake.cook`, `vanilla-cake.cook`
- **test_move_recipe_between_flat_and_nested_category()** → `vanilla-cake.cook`
- **test_move_between_different_category_structures()** → `authors-dinner.cook`

## Implementation Strategy for Tests

When updating tests to use fixtures:

```rust
// Example pattern (before - hardcoded)
let payload = serde_json::json!({
    "name": "Test Recipe",
    "content": "# Test Recipe\n\n@ingredient{} flour",
    "category": "desserts"
});

// After - using fixture
let content = std::fs::read_to_string("tests/fixtures/test-recipe.cook")
    .expect("Failed to read fixture");
let payload = serde_json::json!({
    "name": "Test Recipe",
    "content": content,
    "category": "desserts"
});
```

Or more elegant with a helper function:

```rust
fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{}.cook", name))
        .expect(&format!("Failed to load fixture: {}", name))
}

// Usage
let payload = serde_json::json!({
    "name": "Test Recipe",
    "content": load_fixture("test-recipe"),
    "category": "desserts"
});
```

## Fixture File Naming Convention

- kebab-case filenames: `test-recipe.cook`, `chocolate-cake.cook`
- Matches: `title` field in YAML frontmatter exactly
- Example: `chocolate-cake.cook` contains title "Chocolate Cake"

## Benefits of This Approach

1. **Single Source of Truth**: Each recipe exists in one place
2. **Consistency**: Git and disk storage tests use identical recipes
3. **Maintainability**: Update recipe once, affects all tests using it
4. **Real-world Validation**: Recipes follow production Cooklang schema
5. **Readability**: Test intent becomes clearer without content clutter
