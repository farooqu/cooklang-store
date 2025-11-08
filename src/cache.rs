use dashmap::DashMap;
use std::sync::Arc;

use crate::parser::ScalableRecipe;

/// Represents a recipe in the cache
#[derive(Debug, Clone)]
pub struct CachedRecipe {
    pub git_path: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub recipe: ScalableRecipe,
}

/// In-memory index for fast recipe lookups
pub struct RecipeIndex {
    // Primary index: git_path -> Recipe
    recipes: Arc<DashMap<String, CachedRecipe>>,
}

impl RecipeIndex {
    /// Create a new empty recipe index
    pub fn new() -> Self {
        RecipeIndex {
            recipes: Arc::new(DashMap::new()),
        }
    }

    /// Insert a recipe into the index
    pub fn insert(&self, git_path: String, recipe: CachedRecipe) {
        self.recipes.insert(git_path, recipe);
    }

    /// Get a recipe by git_path
    pub fn get(&self, git_path: &str) -> Option<CachedRecipe> {
        self.recipes.get(git_path).map(|r| r.clone())
    }

    /// Remove a recipe from the index
    pub fn remove(&self, git_path: &str) -> Option<CachedRecipe> {
        self.recipes.remove(git_path).map(|(_, v)| v)
    }

    /// Get all recipes
    pub fn get_all(&self) -> Vec<CachedRecipe> {
        self.recipes
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Search recipes by name (case-insensitive substring match)
    pub fn search_by_name(&self, query: &str) -> Vec<CachedRecipe> {
        let query_lower = query.to_lowercase();
        self.recipes
            .iter()
            .filter(|entry| entry.value().name.to_lowercase().contains(&query_lower))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get recipes by category
    pub fn get_by_category(&self, category: &str) -> Vec<CachedRecipe> {
        self.recipes
            .iter()
            .filter(|entry| entry.value().category.as_deref() == Some(category))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all unique categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();
        for entry in self.recipes.iter() {
            if let Some(cat) = &entry.value().category {
                categories.insert(cat.clone());
            }
        }
        let mut cats: Vec<_> = categories.into_iter().collect();
        cats.sort();
        cats
    }

    /// Filter recipes by ingredient name
    pub fn filter_by_ingredient(&self, ingredient_name: &str) -> Vec<CachedRecipe> {
        let ingredient_lower = ingredient_name.to_lowercase();
        self.recipes
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .recipe
                    .ingredients
                    .iter()
                    .any(|ing| ing.name.to_lowercase().contains(&ingredient_lower))
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get the number of recipes in the index
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Clear all recipes from the index
    pub fn clear(&self) {
        self.recipes.clear();
    }
}

impl Default for RecipeIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RecipeIndex {
    fn clone(&self) -> Self {
        RecipeIndex {
            recipes: Arc::clone(&self.recipes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::CooklangParser;

    fn create_test_recipe(name: &str) -> ScalableRecipe {
        let parser = CooklangParser::new(
            crate::parser::Extensions::all(),
            crate::parser::Converter::default(),
        );
        parser
            .parse(&format!("# {}\n\n@ingredient{{}} test", name), name)
            .into_result()
            .map(|(recipe, _)| recipe)
            .expect("Failed to parse test recipe")
    }

    #[test]
    fn test_insert_and_get() {
        let index = RecipeIndex::new();
        let recipe = CachedRecipe {
            git_path: "recipes/test.cook".to_string(),
            name: "Test Recipe".to_string(),
            description: None,
            category: Some("desserts".to_string()),
            recipe: create_test_recipe("Test Recipe"),
        };

        index.insert("recipes/test.cook".to_string(), recipe.clone());
        let retrieved = index.get("recipes/test.cook").unwrap();

        assert_eq!(retrieved.name, "Test Recipe");
        assert_eq!(retrieved.category, Some("desserts".to_string()));
    }

    #[test]
    fn test_search_by_name() {
        let index = RecipeIndex::new();
        let recipes = vec![
            ("recipes/chocolate.cook", "Chocolate Cake"),
            ("recipes/vanilla.cook", "Vanilla Cake"),
            ("recipes/carrot.cook", "Carrot Cake"),
        ];

        for (path, name) in recipes {
            let recipe = CachedRecipe {
                git_path: path.to_string(),
                name: name.to_string(),
                description: None,
                category: None,
                recipe: create_test_recipe(name),
            };
            index.insert(path.to_string(), recipe);
        }

        let results = index.search_by_name("cake");
        assert_eq!(results.len(), 3);

        let results = index.search_by_name("Chocolate");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Chocolate Cake");
    }

    #[test]
    fn test_get_by_category() {
        let index = RecipeIndex::new();
        let recipes = vec![
            ("recipes/desserts/cake.cook", "Cake", Some("desserts")),
            ("recipes/desserts/brownie.cook", "Brownie", Some("desserts")),
            ("recipes/mains/pasta.cook", "Pasta", Some("mains")),
        ];

        for (path, name, category) in recipes {
            let recipe = CachedRecipe {
                git_path: path.to_string(),
                name: name.to_string(),
                description: None,
                category: category.map(|s| s.to_string()),
                recipe: create_test_recipe(name),
            };
            index.insert(path.to_string(), recipe);
        }

        let desserts = index.get_by_category("desserts");
        assert_eq!(desserts.len(), 2);

        let mains = index.get_by_category("mains");
        assert_eq!(mains.len(), 1);
    }

    #[test]
    fn test_remove() {
        let index = RecipeIndex::new();
        let recipe = CachedRecipe {
            git_path: "recipes/test.cook".to_string(),
            name: "Test".to_string(),
            description: None,
            category: None,
            recipe: create_test_recipe("Test"),
        };

        index.insert("recipes/test.cook".to_string(), recipe);
        assert_eq!(index.len(), 1);

        index.remove("recipes/test.cook");
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_get_categories() {
        let index = RecipeIndex::new();
        let recipes = vec![
            ("recipes/desserts/cake.cook", "Cake", Some("desserts")),
            ("recipes/mains/pasta.cook", "Pasta", Some("mains")),
            ("recipes/appetizers/dip.cook", "Dip", Some("appetizers")),
            ("recipes/uncategorized.cook", "Uncategorized", None),
        ];

        for (path, name, category) in recipes {
            let recipe = CachedRecipe {
                git_path: path.to_string(),
                name: name.to_string(),
                description: None,
                category: category.map(|s| s.to_string()),
                recipe: create_test_recipe(name),
            };
            index.insert(path.to_string(), recipe);
        }

        let categories = index.get_categories();
        assert_eq!(categories.len(), 3);
        assert!(categories.contains(&"desserts".to_string()));
        assert!(categories.contains(&"mains".to_string()));
        assert!(categories.contains(&"appetizers".to_string()));
    }
}
