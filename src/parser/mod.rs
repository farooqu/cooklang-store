use anyhow::{anyhow, Result};
pub use cooklang::{Converter, CooklangParser, Extensions, ScalableRecipe};

pub fn parse_recipe(content: &str, name: &str) -> Result<ScalableRecipe, String> {
    let parser = CooklangParser::new(Extensions::all(), Converter::default());

    parser
        .parse(content, name)
        .into_result()
        .map(|(recipe, _warnings)| recipe)
        .map_err(|report| format!("{}", report))
}

/// Extracts the recipe title from Cooklang content's YAML front matter.
///
/// Expected format:
/// ```text
/// ---
/// title: Recipe Name
/// ---
/// ```
///
/// The function:
/// - Looks for YAML front matter delimited by `---` at the start of content
/// - Parses the YAML block and extracts the `title` field
/// - Returns an error if title is missing or empty
///
/// # Arguments
/// * `content` - The Cooklang recipe content
///
/// # Returns
/// * `Ok(String)` - The extracted recipe title
/// * `Err` - If content is empty, missing front matter, missing title field, or title is empty
///
/// # Examples
/// ```
/// # use cooklang_store::parser::extract_recipe_title;
/// let content = "---\ntitle: Chocolate Cake\n---\n\nRecipe content";
/// let title = extract_recipe_title(content).unwrap();
/// assert_eq!(title, "Chocolate Cake");
/// ```
pub fn extract_recipe_title(content: &str) -> Result<String> {
    let trimmed = content.trim();

    // Check for empty content
    if trimmed.is_empty() {
        return Err(anyhow!("Content is empty"));
    }

    // Check if content starts with front matter delimiter
    if !trimmed.starts_with("---") {
        return Err(anyhow!(
            "Missing YAML front matter: content must start with ---"
        ));
    }

    // Find the closing delimiter
    let remaining = &trimmed[3..]; // Skip opening `---`
    let closing_delimiter_pos = remaining
        .find("---")
        .ok_or_else(|| anyhow!("Malformed YAML front matter: missing closing --- delimiter"))?;

    let front_matter_str = remaining[..closing_delimiter_pos].trim();

    // Parse YAML front matter using serde_yaml
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(front_matter_str)
        .map_err(|e| anyhow!("Invalid YAML front matter: {}", e))?;

    // Extract title field from parsed YAML (case-insensitive key lookup)
    let title_value = yaml_value
        .as_mapping()
        .ok_or_else(|| anyhow!("YAML front matter must be a mapping"))?
        .iter()
        .find(|(key, _)| {
            key.as_str()
                .map(|k| k.to_lowercase() == "title")
                .unwrap_or(false)
        })
        .map(|(_, v)| v)
        .ok_or_else(|| {
            anyhow!(
                "Title field not found in YAML front matter. Expected format: title: Recipe Name"
            )
        })?;

    let title = title_value
        .as_str()
        .ok_or_else(|| anyhow!("Title field must be a string"))?
        .trim();

    if title.is_empty() {
        return Err(anyhow!("Title field is empty in YAML front matter"));
    }

    Ok(title.to_string())
}

/// Generates a filename from a recipe title.
///
/// This function:
/// - Converts the title to lowercase
/// - Replaces spaces and special characters with hyphens
/// - Removes consecutive hyphens, collapsing to a single hyphen
/// - Removes leading and trailing hyphens
/// - Appends the `.cook` extension
///
/// # Arguments
/// * `title` - The recipe title to convert to a filename
///
/// # Returns
/// * `String` - The generated filename (lowercase, hyphen-separated, with .cook extension)
///
/// # Examples
/// ```
/// # use cooklang_store::parser::generate_filename;
/// assert_eq!(generate_filename("Chocolate Cake"), "chocolate-cake.cook");
/// assert_eq!(generate_filename("Quick & Easy Recipe"), "quick-easy-recipe.cook");
/// assert_eq!(generate_filename("5-Ingredient Chili"), "5-ingredient-chili.cook");
/// ```
pub fn generate_filename(title: &str) -> String {
    // Convert to lowercase
    let mut filename = title.to_lowercase();

    // Replace spaces and special characters with hyphens
    // Keep alphanumeric, hyphens, and dots (dots might appear in numbers like "1.5")
    filename = filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                // Replace special characters with hyphens
                '-'
            }
        })
        .collect();

    // Remove consecutive hyphens
    while filename.contains("--") {
        filename = filename.replace("--", "-");
    }

    // Remove leading and trailing hyphens
    filename = filename.trim_matches('-').to_string();

    // Append .cook extension
    format!("{}.cook", filename)
}

/// Normalizes a file path by removing leading/trailing slashes and validating characters.
///
/// This function:
/// - Removes leading and trailing slashes
/// - Validates that the path only contains alphanumeric characters, hyphens, underscores, dots, and slashes
/// - Returns an error if the path contains invalid characters
/// - Returns an error if the path is empty after normalization
///
/// # Arguments
/// * `path` - The file path to normalize
///
/// # Returns
/// * `Ok(String)` - The normalized path
/// * `Err` - If the path contains invalid characters or is empty after normalization
///
/// # Examples
/// ```
/// # use cooklang_store::parser::normalize_path;
/// assert_eq!(normalize_path("/recipes/chocolate-cake.cook").unwrap(), "recipes/chocolate-cake.cook");
/// assert_eq!(normalize_path("recipes/desserts/cake.cook/").unwrap(), "recipes/desserts/cake.cook");
/// ```
pub fn normalize_path(path: &str) -> Result<String> {
    let trimmed = path.trim_matches('/');

    if trimmed.is_empty() {
        return Err(anyhow!("Path is empty after normalization"));
    }

    // Validate characters: allow alphanumeric, hyphens, underscores, dots, and slashes
    for c in trimmed.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' && c != '.' && c != '/' {
            return Err(anyhow!(
                "Path contains invalid character '{}'. Allowed: alphanumeric, hyphens, underscores, dots, slashes",
                c
            ));
        }
    }

    Ok(trimmed.to_string())
}

/// Detects if a file should be renamed based on the old filename and new recipe title.
///
/// This function:
/// - Compares the current filename with the generated filename from the new title
/// - Returns `true` if they differ (rename needed)
/// - Returns `false` if they match (no rename needed)
///
/// This handles cases where:
/// - Recipe title has changed (requires file rename)
/// - File is misaligned with its title (should be corrected on update)
/// - Content structure changes but filename remains correct
///
/// # Arguments
/// * `old_filename` - The current filename (e.g., "chocolate-cake.cook")
/// * `new_title` - The new recipe title (e.g., "Dark Chocolate Cake")
///
/// # Returns
/// * `true` - If the generated filename differs from old filename (rename needed)
/// * `false` - If filenames match (no rename needed)
///
/// # Examples
/// ```
/// # use cooklang_store::parser::should_rename_file;
/// // Title changed: Dark Chocolate Cake vs original Chocolate Cake
/// assert!(should_rename_file("chocolate-cake.cook", "Dark Chocolate Cake"));
///
/// // Title unchanged: same generated filename
/// assert!(!should_rename_file("chocolate-cake.cook", "Chocolate Cake"));
///
/// // Title spacing differs but generates same filename
/// assert!(!should_rename_file("chocolate-cake.cook", "  Chocolate   Cake  "));
/// ```
pub fn should_rename_file(old_filename: &str, new_title: &str) -> bool {
    let generated_filename = generate_filename(new_title);
    generated_filename != old_filename
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_recipe() {
        let content = r#">> name: Scrambled Eggs
>> servings: 2

Crack @eggs{2} into a bowl and whisk. Heat @butter{1%tbsp} in a #pan over medium heat. Pour in the eggs and cook for ~{2%minutes}, stirring constantly.
"#;

        let result = parse_recipe(content, "Scrambled Eggs");
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(recipe.name, "Scrambled Eggs");
        assert_eq!(recipe.metadata.map.get("servings"), Some(&"2".to_string()));
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.cookware.len(), 1);
        assert_eq!(recipe.timers.len(), 1);
    }

    #[test]
    fn test_parse_ingredient_with_quantity() {
        let content = r#"Add @flour{2%cups} to the bowl."#;

        let result = parse_recipe(content, "Test Recipe");
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(recipe.ingredients[0].name, "flour");
        assert!(recipe.ingredients[0].quantity.is_some());
    }

    #[test]
    fn test_parse_multiple_steps() {
        let content = r#"First step.

Second step.

Third step."#;

        let result = parse_recipe(content, "Test Recipe");
        assert!(result.is_ok());

        let recipe = result.unwrap();
        let total_steps: usize = recipe.sections.iter().map(|s| s.steps.len()).sum();
        assert_eq!(total_steps, 3);
    }

    #[test]
    fn test_parse_with_metadata() {
        let content = r#">> author: John Doe
>> tags: breakfast, quick

Add @eggs{2} to the bowl."#;

        let result = parse_recipe(content, "Test Recipe");
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(
            recipe.metadata.map.get("author"),
            Some(&"John Doe".to_string())
        );
        assert_eq!(
            recipe.metadata.map.get("tags"),
            Some(&"breakfast, quick".to_string())
        );
    }

    // Tests for extract_recipe_title
    #[test]
    fn test_extract_title_standard_format() {
        let content = "---\ntitle: Chocolate Cake\n---\n\nRecipe content here";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_with_whitespace() {
        let content = "---\ntitle:   Chocolate Cake   \n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_case_insensitive_key() {
        let content = "---\nTitle: Chocolate Cake\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_uppercase_key() {
        let content = "---\nTITLE: Chocolate Cake\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_with_double_quotes() {
        let content = "---\ntitle: \"Chocolate Cake\"\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_with_single_quotes() {
        let content = "---\ntitle: 'Chocolate Cake'\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_with_multiple_fields() {
        let content =
            "---\nauthor: John Doe\ntitle: Chocolate Cake\ntags: dessert\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_with_comments() {
        let content = "---\n# This is a comment\ntitle: Chocolate Cake\n# Another comment\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_with_empty_lines() {
        let content = "---\n\nauthor: John Doe\n\ntitle: Chocolate Cake\n\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Chocolate Cake");
    }

    #[test]
    fn test_extract_title_special_characters() {
        let content = "---\ntitle: Dark Chocolate & Vanilla Layer Cake\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Dark Chocolate & Vanilla Layer Cake");
    }

    #[test]
    fn test_extract_title_with_numbers() {
        let content = "---\ntitle: 5-Ingredient Chocolate Cake\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5-Ingredient Chocolate Cake");
    }

    #[test]
    fn test_extract_title_empty_content() {
        let content = "";
        let result = extract_recipe_title(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Content is empty"));
    }

    #[test]
    fn test_extract_title_missing_front_matter() {
        let content = "No front matter here\ntitle: Chocolate Cake";
        let result = extract_recipe_title(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing YAML front matter"));
    }

    #[test]
    fn test_extract_title_missing_closing_delimiter() {
        let content = "---\ntitle: Chocolate Cake\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing closing --- delimiter"));
    }

    #[test]
    fn test_extract_title_missing_title_field() {
        let content = "---\nauthor: John Doe\ntags: dessert\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Title field not found"));
    }

    #[test]
    fn test_extract_title_empty_title_value() {
        let content = "---\ntitle:\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_err());
        // serde_yaml parses empty value as null, so we get "must be a string" error
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Title field must be a string"));
    }

    #[test]
    fn test_extract_title_only_whitespace() {
        let content = "   \n   \n   ";
        let result = extract_recipe_title(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Content is empty"));
    }

    #[test]
    fn test_extract_title_unicode_characters() {
        let content = "---\ntitle: Gâteau Chocolat Français\n---\n\nRecipe content";
        let result = extract_recipe_title(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Gâteau Chocolat Français");
    }

    #[test]
    fn test_extract_title_long_name() {
        let long_title = "The Most Delicious and Decadent Triple-Layer Chocolate Cake with \
                         Ganache Frosting and Fresh Berries on Top";
        let content = format!("---\ntitle: {}\n---\n\nRecipe content", long_title);
        let result = extract_recipe_title(&content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), long_title);
    }

    // Tests for generate_filename
    #[test]
    fn test_generate_filename_simple_title() {
        assert_eq!(generate_filename("Chocolate Cake"), "chocolate-cake.cook");
    }

    #[test]
    fn test_generate_filename_already_lowercase() {
        assert_eq!(generate_filename("pasta carbonara"), "pasta-carbonara.cook");
    }

    #[test]
    fn test_generate_filename_all_uppercase() {
        assert_eq!(generate_filename("BEEF STEAK"), "beef-steak.cook");
    }

    #[test]
    fn test_generate_filename_mixed_case() {
        assert_eq!(
            generate_filename("Chocolate Layer Cake"),
            "chocolate-layer-cake.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_special_characters() {
        assert_eq!(
            generate_filename("Quick & Easy Recipe"),
            "quick-easy-recipe.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_numbers() {
        assert_eq!(
            generate_filename("5-Ingredient Chili"),
            "5-ingredient-chili.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_apostrophe() {
        assert_eq!(
            generate_filename("Mom's Secret Sauce"),
            "mom-s-secret-sauce.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_parentheses() {
        assert_eq!(generate_filename("Pasta (Homemade)"), "pasta-homemade.cook");
    }

    #[test]
    fn test_generate_filename_with_multiple_spaces() {
        assert_eq!(
            generate_filename("Chocolate    Cake"),
            "chocolate-cake.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_leading_trailing_spaces() {
        assert_eq!(
            generate_filename("  Chocolate Cake  "),
            "chocolate-cake.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_special_chars_consecutive() {
        assert_eq!(
            generate_filename("Cake & Frosting & Berries"),
            "cake-frosting-berries.cook"
        );
    }

    #[test]
    fn test_generate_filename_unicode_characters() {
        assert_eq!(
            generate_filename("Gâteau Chocolat Français"),
            "gâteau-chocolat-français.cook"
        );
    }

    #[test]
    fn test_generate_filename_with_dots_in_numbers() {
        assert_eq!(
            generate_filename("1.5 Liter Smoothie"),
            "1.5-liter-smoothie.cook"
        );
    }

    #[test]
    fn test_generate_filename_single_word() {
        assert_eq!(generate_filename("Brownies"), "brownies.cook");
    }

    #[test]
    fn test_generate_filename_with_hyphens() {
        assert_eq!(
            generate_filename("Chocolate-Chip Cookies"),
            "chocolate-chip-cookies.cook"
        );
    }

    #[test]
    fn test_generate_filename_empty_title() {
        assert_eq!(generate_filename(""), ".cook");
    }

    #[test]
    fn test_generate_filename_only_special_characters() {
        assert_eq!(generate_filename("&&&"), ".cook");
    }

    #[test]
    fn test_generate_filename_very_long_title() {
        let long_title = "The Most Delicious and Decadent Triple-Layer Chocolate Cake with Ganache";
        let result = generate_filename(long_title);
        assert!(result.ends_with(".cook"));
        assert!(result
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '.'));
    }

    // Tests for normalize_path
    #[test]
    fn test_normalize_path_simple() {
        assert_eq!(
            normalize_path("recipes/chocolate-cake.cook").unwrap(),
            "recipes/chocolate-cake.cook"
        );
    }

    #[test]
    fn test_normalize_path_with_leading_slash() {
        assert_eq!(
            normalize_path("/recipes/chocolate-cake.cook").unwrap(),
            "recipes/chocolate-cake.cook"
        );
    }

    #[test]
    fn test_normalize_path_with_trailing_slash() {
        assert_eq!(
            normalize_path("recipes/chocolate-cake.cook/").unwrap(),
            "recipes/chocolate-cake.cook"
        );
    }

    #[test]
    fn test_normalize_path_with_leading_and_trailing_slashes() {
        assert_eq!(
            normalize_path("/recipes/chocolate-cake.cook/").unwrap(),
            "recipes/chocolate-cake.cook"
        );
    }

    #[test]
    fn test_normalize_path_nested_directories() {
        assert_eq!(
            normalize_path("/recipes/desserts/cakes/chocolate-cake.cook").unwrap(),
            "recipes/desserts/cakes/chocolate-cake.cook"
        );
    }

    #[test]
    fn test_normalize_path_with_underscores() {
        assert_eq!(
            normalize_path("recipes/my_recipe.cook").unwrap(),
            "recipes/my_recipe.cook"
        );
    }

    #[test]
    fn test_normalize_path_with_numbers() {
        assert_eq!(
            normalize_path("recipes/5-ingredient-chili.cook").unwrap(),
            "recipes/5-ingredient-chili.cook"
        );
    }

    #[test]
    fn test_normalize_path_single_file() {
        assert_eq!(normalize_path("recipe.cook").unwrap(), "recipe.cook");
    }

    #[test]
    fn test_normalize_path_empty_string() {
        assert!(normalize_path("").is_err());
        assert!(normalize_path("")
            .unwrap_err()
            .to_string()
            .contains("empty"));
    }

    #[test]
    fn test_normalize_path_only_slashes() {
        assert!(normalize_path("///").is_err());
        assert!(normalize_path("///")
            .unwrap_err()
            .to_string()
            .contains("empty"));
    }

    #[test]
    fn test_normalize_path_with_spaces() {
        assert!(normalize_path("recipes/chocolate cake.cook").is_err());
        assert!(normalize_path("recipes/chocolate cake.cook")
            .unwrap_err()
            .to_string()
            .contains("invalid character"));
    }

    #[test]
    fn test_normalize_path_with_special_characters() {
        assert!(normalize_path("recipes/cake&frosting.cook").is_err());
        assert!(normalize_path("recipes/cake@frosting.cook").is_err());
        assert!(normalize_path("recipes/cake#frosting.cook").is_err());
    }

    #[test]
    fn test_normalize_path_with_dots_in_filename() {
        assert_eq!(
            normalize_path("recipes/1.5.liter.smoothie.cook").unwrap(),
            "recipes/1.5.liter.smoothie.cook"
        );
    }

    #[test]
    fn test_normalize_path_mixed_valid_chars() {
        assert_eq!(
            normalize_path("/recipes/desserts/my_5-ingredient_cake.cook/").unwrap(),
            "recipes/desserts/my_5-ingredient_cake.cook"
        );
    }

    // Tests for should_rename_file
    #[test]
    fn test_should_rename_file_title_changed() {
        // Title changed from "Chocolate Cake" to "Dark Chocolate Cake"
        assert!(should_rename_file(
            "chocolate-cake.cook",
            "Dark Chocolate Cake"
        ));
    }

    #[test]
    fn test_should_rename_file_title_unchanged() {
        // Title remains the same, no rename needed
        assert!(!should_rename_file("chocolate-cake.cook", "Chocolate Cake"));
    }

    #[test]
    fn test_should_rename_file_title_with_extra_whitespace() {
        // Title has different whitespace but generates same filename
        assert!(!should_rename_file(
            "chocolate-cake.cook",
            "  Chocolate   Cake  "
        ));
    }

    #[test]
    fn test_should_rename_file_case_difference() {
        // Case differs but generates same lowercase filename
        assert!(!should_rename_file("chocolate-cake.cook", "CHOCOLATE CAKE"));
    }

    #[test]
    fn test_should_rename_file_special_chars_removed() {
        // Special characters are normalized away, resulting in same filename
        assert!(!should_rename_file(
            "quick-easy-recipe.cook",
            "Quick & Easy Recipe"
        ));
    }

    #[test]
    fn test_should_rename_file_multiple_spaces_to_hyphens() {
        // Multiple spaces normalized to single hyphens
        assert!(!should_rename_file(
            "chocolate-cake.cook",
            "Chocolate    Cake"
        ));
    }

    #[test]
    fn test_should_rename_file_leading_trailing_spaces() {
        // Leading/trailing spaces are trimmed during filename generation
        assert!(!should_rename_file(
            "chocolate-cake.cook",
            "  Chocolate Cake  "
        ));
    }

    #[test]
    fn test_should_rename_file_word_removed() {
        // One word removed from title
        assert!(should_rename_file(
            "chocolate-layer-cake.cook",
            "Chocolate Cake"
        ));
    }

    #[test]
    fn test_should_rename_file_word_added() {
        // New word added to title
        assert!(should_rename_file(
            "chocolate-cake.cook",
            "Decadent Chocolate Cake"
        ));
    }

    #[test]
    fn test_should_rename_file_word_order_changed() {
        // Words reordered changes the filename
        assert!(should_rename_file("chocolate-cake.cook", "Cake Chocolate"));
    }

    #[test]
    fn test_should_rename_file_apostrophe_handling() {
        // Apostrophe normalized to hyphen
        assert!(!should_rename_file(
            "mom-s-secret-sauce.cook",
            "Mom's Secret Sauce"
        ));
    }

    #[test]
    fn test_should_rename_file_parentheses_removed() {
        // Parentheses and content normalized away
        assert!(!should_rename_file(
            "pasta-homemade.cook",
            "Pasta (Homemade)"
        ));
    }

    #[test]
    fn test_should_rename_file_unicode_chars() {
        // Unicode characters preserved (lowercase transformation)
        assert!(!should_rename_file(
            "gâteau-chocolat-français.cook",
            "Gâteau Chocolat Français"
        ));
    }

    #[test]
    fn test_should_rename_file_numbers_in_title() {
        // Numbers preserved in filename
        assert!(!should_rename_file(
            "5-ingredient-chili.cook",
            "5-Ingredient Chili"
        ));
    }

    #[test]
    fn test_should_rename_file_dots_in_title() {
        // Dots preserved in filenames (e.g., "1.5")
        assert!(!should_rename_file(
            "1.5-liter-smoothie.cook",
            "1.5 Liter Smoothie"
        ));
    }

    #[test]
    fn test_should_rename_file_significant_title_change() {
        // Completely different title
        assert!(should_rename_file("chocolate-cake.cook", "Vanilla Pudding"));
    }

    #[test]
    fn test_should_rename_file_single_character_difference() {
        // Single character change in title
        assert!(should_rename_file("chocolate-cake.cook", "Chocolate Cakee"));
    }

    #[test]
    fn test_should_rename_file_empty_old_filename() {
        // Edge case: empty old filename (should rename if new title generates non-empty)
        assert!(should_rename_file("", "Chocolate Cake"));
    }

    #[test]
    fn test_should_rename_file_both_empty() {
        // Both empty or generate empty filenames
        assert!(!should_rename_file(".cook", ""));
    }
}
