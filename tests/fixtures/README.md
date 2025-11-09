# Test Recipe Fixtures

This directory contains properly formatted Cooklang recipe files used in integration tests. Each recipe follows the Cooklang specification with YAML frontmatter and inline ingredient usage.

## Cooklang Format

Each recipe file follows this structure:

```cooklang
---
title: Recipe Name
description: Brief description of the recipe
---

First paragraph as first step, with @ingredient{amount%unit} used inline.

Second paragraph as second step, continuing to use @ingredient{} directly in the instructions.

Each blank line separated paragraph becomes a distinct step in the recipe.
```

Key characteristics:
- **No markdown headers** - Just plain paragraphs
- **No separate ingredient lists** - All ingredients are used inline with @ingredient{amount%unit} syntax
- **No numbered steps** - Each paragraph naturally becomes a step
- **YAML frontmatter** - Required title and description fields
- **Ingredients inline** - Used directly in instructions with proper amounts and units

## Recipe Inventory (19 Total)

### Basic Recipes (8)
- **test-recipe.cook** - Simple test recipe with basic ingredient
- **chocolate-cake.cook** - Dessert with multiple ingredients
- **recipe-1.cook** - Numbered recipe for pagination tests
- **recipe-2.cook** - Numbered recipe for pagination tests
- **cake.cook** - Simple cake for category tests
- **pasta.cook** - Main course recipe
- **to-delete.cook** - Recipe for deletion operation tests
- **vanilla-cake.cook** - Vanilla dessert variant

### Update/Lifecycle Recipes (2)
- **original-name.cook** - Original state before updates
- **updated-name.cook** - Updated state after modifications

### Nested Category Recipes (9)
- **chicken-biryani.cook** - Indian rice dish
- **thai-green-curry.cook** - Thai curry with coconut milk
- **pad-thai.cook** - Thai noodle dish
- **green-curry.cook** - Simpler curry version
- **spaghetti.cook** - Italian pasta
- **tiramisu.cook** - Italian dessert
- **cheesecake.cook** - American dessert
- **flan.cook** - Spanish custard dessert
- **authors-dinner.cook** - Braised beef (complex category movement testing)

## Format Validation

All 19 recipes have been validated:
- ✓ YAML frontmatter present (title and description)
- ✓ No markdown formatting in recipe body
- ✓ All ingredients used inline with @ingredient syntax
- ✓ No numbered steps - paragraph-based structure
- ✓ Proper Cooklang syntax throughout

## Usage in Tests

When tests load these fixtures, they will:

1. Read the complete file content
2. Send it as the `content` field in API requests
3. The parser will extract the title from the recipe content (not relying on separate name field)
4. Both git and disk storage tests use the same fixture files

Example test pattern:
```rust
let content = std::fs::read_to_string("tests/fixtures/chocolate-cake.cook")
    .expect("Failed to read fixture");
let payload = serde_json::json!({
    "name": "Chocolate Cake",  // Matches the title in the recipe
    "content": content,
    "category": "desserts"
});
```

## Total Count

19 distinct test recipes providing coverage for:
- Basic CRUD operations
- Pagination and filtering
- Search functionality
- Flat and nested category structures
- Category movement operations
- Update operations (rename and category change)
- Deletion operations
