pub use cooklang::{Converter, CooklangParser, Extensions, ScalableRecipe};

pub fn parse_recipe(content: &str, name: &str) -> Result<ScalableRecipe, String> {
    let parser = CooklangParser::new(Extensions::all(), Converter::default());

    parser
        .parse(content, name)
        .into_result()
        .map(|(recipe, _warnings)| recipe)
        .map_err(|report| format!("{}", report))
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
}
