use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cooklang_backend::{api, repository::RecipeRepository};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if it exists
    if let Ok(env_file) = std::fs::read_to_string(".env") {
        for line in env_file.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                if !key.starts_with('#') && !key.is_empty() {
                    std::env::set_var(key, value);
                }
            }
        }
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cooklang_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize repository from .cooklang directory
    let repo_path = Path::new(".cooklang");
    let storage_type =
        std::env::var("COOKLANG_STORAGE_TYPE").unwrap_or_else(|_| "disk".to_string());

    let repo = match RecipeRepository::with_storage(repo_path, &storage_type).await {
        Ok(repo) => {
            tracing::info!(
                "Initialized recipe repository at {:?} with storage type: {}",
                repo_path,
                storage_type
            );
            Arc::new(repo)
        }
        Err(e) => {
            tracing::error!("Failed to initialize repository: {}", e);
            std::process::exit(1);
        }
    };

    // Build the app with the repository
    let app = api::build_router(repo);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
