# Sample Recipes for Testing

This document contains sample recipes that can be used to test the Cooklang Store API.

## How to Use

Copy any recipe content below and use it in API requests via curl, Postman, or your preferred API client.

### Quick Test Script

```bash
# Set your base URL
BASE_URL="http://localhost:3000"

# Create a recipe
curl -X POST $BASE_URL/api/v1/recipes \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pasta Carbonara",
    "content": "# Pasta Carbonara\n\n@eggs{4} @bacon{200%g} @pasta{400%g}",
    "category": "mains",
    "author": "Test User"
  }'

# List recipes
curl $BASE_URL/api/v1/recipes

# Search recipes
curl "$BASE_URL/api/v1/recipes/search?q=pasta"

# List categories
curl $BASE_URL/api/v1/categories
```

## Sample Recipe 1: Pasta Carbonara

**Category**: mains  
**Servings**: 4 people  
**Prep Time**: 15 minutes  
**Cook Time**: 20 minutes

```
# Pasta Carbonara

Classic Italian pasta dish with eggs, bacon, and cheese.

>> servings: 4
>> prep_time: 15 minutes
>> cook_time: 20 minutes

Bring a large #pot of @water{4%liters} to boil, then add @salt{2%teaspoons} ~{8-10%minutes}.

While water heats, cut @bacon{200%g} into small pieces and cook in a #pan over medium heat until crispy ~{5%minutes}.

In a large bowl, whisk together @eggs{4}, @egg_yolks{2}, @parmesan{100%g}, @black_pepper{1%teaspoon}, and a pinch of @salt{to taste}.

Add @pasta{400%g} to the boiling water and cook according to package directions until al dente ~{8-10%minutes}.

Reserve 1 #cup of pasta water before draining.

Add the hot drained pasta to the bacon in the #pan and toss over low heat.

Remove from heat and stir in the egg mixture, adding pasta water a little at a time until you reach the desired consistency.

Serve immediately, garnished with more @parmesan and @black_pepper.
```

## Sample Recipe 2: Chocolate Chip Cookies

**Category**: desserts  
**Servings**: 24 cookies  
**Prep Time**: 15 minutes  
**Bake Time**: 12 minutes

```
# Chocolate Chip Cookies

Classic homemade chocolate chip cookies.

>> servings: 24 cookies
>> prep_time: 15 minutes
>> bake_time: 12 minutes

Preheat oven to 375°F (190°C).

In a small bowl, mix @flour{2.25%cups}, @baking_soda{1%teaspoon}, and @salt{1%teaspoon}.

In a large #bowl, beat @butter{1%cup} with @brown_sugar{0.75%cup} and @granulated_sugar{0.75%cup} until creamy ~{3%minutes}.

Add @eggs{2} and @vanilla_extract{1%teaspoon} to the butter mixture and beat well.

Gradually blend in the flour mixture using a #mixer.

Stir in @chocolate_chips{2%cups}.

Drop rounded tablespoons of dough onto ungreased #baking_sheets, spacing about 2 inches apart.

Bake for 12 minutes until golden brown ~{12%minutes}.

Cool on the #baking_sheet for 2 minutes, then transfer to a #wire_rack.

Makes about 24 cookies.
```

## Sample Recipe 3: Vegetable Stir-Fry

**Category**: mains  
**Servings**: 2 people  
**Prep Time**: 20 minutes  
**Cook Time**: 10 minutes

```
# Quick Vegetable Stir-Fry

A healthy, quick weeknight dinner packed with vegetables.

>> servings: 2 people
>> prep_time: 20 minutes
>> cook_time: 10 minutes

Heat @oil{2%tablespoons} in a large #wok or #skillet over high heat.

Add @garlic{2%cloves, minced} and @ginger{1%teaspoon, minced} and stir-fry for about 30 seconds until fragrant.

Add @broccoli{2%cups, florets} and @bell_peppers{1%large, sliced} and stir-fry for 2-3 minutes.

Add @snap_peas{1%cup}, @carrots{1%medium, sliced}, and @mushrooms{1%cup, sliced} and continue stir-frying for another 2-3 minutes.

In a small bowl, whisk together @soy_sauce{3%tablespoons}, @rice_vinegar{1%tablespoon}, @sesame_oil{1%teaspoon}, and @corn_starch{1%teaspoon}.

Pour the sauce into the #wok and toss everything to coat. Stir-fry for 1-2 minutes until sauce thickens.

Serve over @rice{1%cup, cooked} or @noodles{1%cup, cooked}.

Optional: Top with @sesame_seeds{1%tablespoon} and @green_onions{2, chopped}.
```

## Sample Recipe 4: Tomato Soup

**Category**: appetizers  
**Servings**: 4 people  
**Prep Time**: 10 minutes  
**Cook Time**: 30 minutes

```
# Creamy Tomato Soup

Comfort food in a bowl - smooth, creamy tomato soup.

>> servings: 4 people
>> prep_time: 10 minutes
>> cook_time: 30 minutes

Heat @butter{2%tablespoons} in a large #pot over medium heat.

Sauté @onion{1%large, diced} and @garlic{3%cloves, minced} until soft ~{5%minutes}.

Add @tomatoes_canned{2%cans, 400g each}, @vegetable_broth{2%cups}, @salt{1%teaspoon}, and @black_pepper{0.5%teaspoon}.

Bring to a boil, then reduce heat and simmer ~{15%minutes}.

Using an immersion #blender, blend until smooth. (Alternatively, carefully blend in batches using a regular blender.)

Stir in @heavy_cream{0.5%cup} and heat through without boiling ~{5%minutes}.

Optional: Stir in fresh @basil{2%tablespoons, chopped} just before serving.

Serve hot with crusty bread or croutons.
```

## Sample Recipe 5: Grilled Salmon

**Category**: mains  
**Servings**: 2 people  
**Prep Time**: 10 minutes  
**Cook Time**: 12 minutes

```
# Grilled Lemon Herb Salmon

Simple and elegant grilled salmon with fresh herbs.

>> servings: 2 people
>> prep_time: 10 minutes
>> cook_time: 12 minutes

Preheat grill to medium-high heat (about 200°C / 400°F).

Pat @salmon_fillets{2, 150g each} dry with paper towels.

Brush both sides with @olive_oil{1%tablespoon} and season with @salt and @black_pepper to taste.

Zest @lemon{1} and chop fresh @dill{1%tablespoon} and @parsley{1%tablespoon}.

Place salmon skin-side up on the grill and cook ~{6%minutes}.

Carefully flip and cook for another ~{4-6%minutes} until cooked through.

Transfer to plates and top with lemon zest, fresh herbs, and a squeeze of @lemon_juice{1%tablespoon}.

Serve immediately with your choice of @rice, @potatoes, or @vegetables.
```

## Testing with Postman

1. Import the `postman-collection.json` file into Postman
2. Set the `base_url` variable to your server URL (e.g., `http://localhost:3000`)
3. Use the sample recipe content from above in the request bodies
4. Execute the requests in order:
   - Create Recipe
   - List Recipes
   - Get Recipe (uses the ID from Create)
   - Update Recipe
   - Delete Recipe

## Testing with curl

Create a recipe:
```bash
curl -X POST http://localhost:3000/api/v1/recipes \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pasta Carbonara",
    "content": "# Pasta Carbonara\n\n@eggs{4} @bacon{200%g}",
    "category": "mains"
  }'
```

Search for recipes:
```bash
curl "http://localhost:3000/api/v1/recipes/search?q=pasta"
```

Get categories:
```bash
curl http://localhost:3000/api/v1/categories
```

## Cooklang Syntax Reference

- **Ingredients**: `@ingredient{quantity%unit}` or just `@ingredient{quantity}`
- **Cookware**: `#cookware` (marks items needed for cooking)
- **Timers**: `~{duration%unit}` (marks cooking times)
- **Metadata**: Lines starting with `>>` at the beginning of file
- **Comments**: `-- comment` or `[- comment -]`

For the complete Cooklang specification, see: https://cooklang.org/docs/spec/
