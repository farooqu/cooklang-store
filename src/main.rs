use clap::Parser;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cooklang_backend::{api, repository::RecipeRepository};

#[derive(Parser)]
#[command(name = "cooklang-store")]
#[command(about = "A self-hosted service for managing Cooklang recipe files", long_about = None)]
struct Args {
    /// Path to the data directory containing recipes
    #[arg(short, long, required = true)]
    data_dir: String,

    /// Storage type (disk or git)
    #[arg(short, long, default_value = "disk")]
    storage: String,
}

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

    // Parse command-line arguments
    let args = Args::parse();

    let repo_path = Path::new(&args.data_dir);

    let repo = match RecipeRepository::with_storage(repo_path, &args.storage).await {
        Ok(repo) => {
            tracing::info!(
                "Initialized recipe repository at {:?} with storage type: {}",
                repo_path,
                args.storage
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
