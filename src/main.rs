use std::sync::Arc;
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cooklang_backend::{api, repository::RecipeRepository};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cooklang_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize repository from .cooklang directory
    let repo_path = Path::new(".cooklang");
    let repo = match RecipeRepository::new(repo_path).await {
        Ok(repo) => {
            tracing::info!("Initialized recipe repository at {:?}", repo_path);
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
