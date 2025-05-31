use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

mod config;
mod error;
mod handlers;
mod models;
mod services;
mod utils;

use config::Config;
use models::BlogPost;
use services::BlogService;
use utils::TemplateEngine;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub posts: Vec<BlogPost>,
    pub template_engine: TemplateEngine,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    tracing::info!("tracing is initialized");

    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {}, using defaults", e);
        Config::default()
    });

    // Load blog posts
    let posts = BlogService::load_posts().unwrap_or_else(|e| {
        tracing::error!("Failed to load blog posts: {}", e);
        Vec::new()
    });

    tracing::info!("Loaded {} blog posts", posts.len());

    // Initialize template engine
    let template_engine = TemplateEngine::new(config.clone());

    // Create application state
    let state = AppState {
        config: config.clone(),
        posts,
        template_engine,
    };

    // Build router
    let router: Router = Router::new()
        .route("/", get(handlers::serve_index))
        .route("/blog", get(handlers::serve_blog))
        .route("/blog/:post", get(handlers::serve_blog_post))
        .route("/feed.xml", get(handlers::serve_rss_feed))
        .route("/sitemap.xml", get(handlers::serve_sitemap))
        .route("/robots.txt", get(handlers::serve_robots_txt))
        .route("/manifest.json", get(handlers::serve_manifest))
        .fallback(handlers::serve_404)
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CompressionLayer::new()
                        .br(true)
                        .gzip(true)
                        .deflate(true)
                        .quality(CompressionLevel::Best),
                ),
        );

    // Start server
    let bind_address = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting server on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
