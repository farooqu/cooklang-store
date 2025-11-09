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
/// ```
/// ---
/// title: Recipe Name
/// ---
/// ```
///
/// The function:
/// - Looks for YAML front matter delimited by `---` at the start of content
/// - Extracts the `title` field (case-insensitive key lookup)
/// - Returns an error if title is missing, empty, or front matter is malformed
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
/// let content = "---\ntitle: Chocolate Cake\n---\n\n# Recipe content";
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

    let front_matter = &remaining[..closing_delimiter_pos].trim();

    // Parse the front matter to extract the title field
    let title = extract_title_from_yaml(front_matter)?;

    Ok(title)
}

/// Parses YAML front matter and extracts the title field (case-insensitive).
///
/// YAML parsing is simple: looks for a line matching `title: <value>` (case-insensitive key).
fn extract_title_from_yaml(yaml_content: &str) -> Result<String> {
    for line in yaml_content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Look for key: value pattern
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim();

            if key == "title" {
                if value.is_empty() {
                    return Err(anyhow!("Title field is empty in YAML front matter"));
                }

                // Handle quoted values (remove surrounding quotes if present)
                let title = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                return Ok(title.to_string());
            }
        }
    }

    Err(anyhow!(
        "Title field not found in YAML front matter. Expected format: title: Recipe Name"
    ))
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Title field is empty"));
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
}
