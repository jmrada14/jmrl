use crate::{
    error::Result,
    services::{BlogService, RssService, SitemapService},
    AppState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;

#[tracing::instrument(name = "index", skip(state))]
pub async fn serve_index(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("serving index");

    let context = HashMap::new();
    let html = state
        .template_engine
        .render_template("templates/index.html", &context)
        .or_else(|_| {
            state
                .template_engine
                .render_template("assets/index.html", &context)
        })?;

    Ok(Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_html())
        .header("X-Content-Type-Options", "nosniff")
        .body(html)
        .unwrap())
}

#[tracing::instrument(name = "blog", skip(state))]
pub async fn serve_blog(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("serving blog page");

    let blog_content = state.template_engine.render_blog_list(&state.posts);
    let mut context = HashMap::new();
    context.insert("BLOG_POSTS".to_string(), blog_content);

    let html = state
        .template_engine
        .render_template("templates/blog.html", &context)
        .or_else(|_| {
            state
                .template_engine
                .render_template("assets/blog.html", &context)
        })?;

    Ok(Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_html())
        .header("X-Content-Type-Options", "nosniff")
        .body(html)
        .unwrap())
}

#[tracing::instrument(name = "blog_post", fields(post_slug = %post_slug), skip(state))]
pub async fn serve_blog_post(
    State(state): State<AppState>,
    Path(post_slug): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("serving blog post: {}", post_slug);

    let post = BlogService::get_post_by_slug(&state.posts, &post_slug).ok_or_else(|| {
        crate::error::AppError::NotFound(format!("Post not found: {}", post_slug))
    })?;

    tracing::debug!("found post: {}", post.title);

    let mut context = HashMap::new();
    context.insert("BLOG_TITLE".to_string(), post.title.clone());
    context.insert("BLOG_DATE".to_string(), post.formatted_date());
    context.insert("BLOG_SLUG".to_string(), post_slug.clone());

    // Use description for meta tags instead of HTML excerpt
    let meta_excerpt = if post.description.is_empty() {
        post.title.clone()
    } else {
        post.description.clone()
    };
    context.insert("BLOG_EXCERPT".to_string(), meta_excerpt);

    context.insert(
        "BLOG_TAGS".to_string(),
        state.template_engine.render_post_tags(&post.tags),
    );
    context.insert(
        "BLOG_TAGS_PLAIN".to_string(),
        state.template_engine.render_post_tags_plain(&post.tags),
    );
    context.insert("READING_TIME".to_string(), post.reading_time.to_string());
    context.insert("BLOG_CONTENT".to_string(), post.content.clone());

    let html = state
        .template_engine
        .render_template("templates/post.html", &context)
        .or_else(|_| {
            state
                .template_engine
                .render_template("assets/post.html", &context)
        })?;

    Ok(Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_html())
        .header("X-Content-Type-Options", "nosniff")
        .body(html)
        .unwrap())
}

#[tracing::instrument(name = "rss_feed", skip(state))]
pub async fn serve_rss_feed(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("serving RSS feed");

    let rss_content = RssService::generate_feed(&state.config, &state.posts);

    Ok(Response::builder()
        .header("Content-Type", "application/rss+xml; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_html())
        .body(rss_content)
        .unwrap())
}

#[tracing::instrument(name = "sitemap", skip(state))]
pub async fn serve_sitemap(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("serving sitemap");

    let sitemap_content = SitemapService::generate_sitemap(&state.config, &state.posts);

    Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_html())
        .body(sitemap_content)
        .unwrap())
}

#[tracing::instrument(name = "robots", skip(state))]
pub async fn serve_robots_txt(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("serving robots.txt");

    // Try to read from static file first, fallback to basic content
    let robots_content = match std::fs::read_to_string("static/robots.txt") {
        Ok(content) => {
            // Replace domain placeholder if needed
            content.replace(
                "https://jrada.dev",
                &format!("https://{}", state.config.site.domain),
            )
        }
        Err(_) => {
            // Fallback to basic robots.txt
            format!(
                "User-agent: *\nAllow: /\n\nSitemap: https://{}/sitemap.xml\n",
                state.config.site.domain
            )
        }
    };

    Ok(Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_static())
        .body(robots_content)
        .unwrap())
}

#[tracing::instrument(name = "manifest", skip(state))]
pub async fn serve_manifest(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::info!("serving manifest.json");

    // Read manifest.json from static files
    let manifest_content = std::fs::read_to_string("static/manifest.json").unwrap_or_else(|_| {
        // Fallback manifest if file doesn't exist
        format!(
            "{{
  \"name\": \"{}\",
  \"short_name\": \"{}\",
  \"description\": \"{}\",
  \"start_url\": \"/\",
  \"display\": \"standalone\",
  \"background_color\": \"#ffffff\",
  \"theme_color\": \"#2563eb\",
  \"icons\": []
}}",
            state.config.site.title,
            state
                .config
                .site
                .title
                .split_whitespace()
                .next()
                .unwrap_or("Site"),
            state.config.site.description
        )
    });

    Ok(Response::builder()
        .header("Content-Type", "application/manifest+json; charset=utf-8")
        .header("Cache-Control", state.config.cache_control_static())
        .body(manifest_content)
        .unwrap())
}

#[tracing::instrument(name = "not_found", skip(state))]
pub async fn serve_404(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::warn!("serving 404 page");

    let context = HashMap::new();
    let html = state.template_engine.render_template("templates/404.html", &context)
        .unwrap_or_else(|_| {
            // Fallback 404 page
            "<!DOCTYPE html><html><head><title>404 - Page Not Found</title></head><body><h1>404 - Page Not Found</h1><p>The requested page could not be found.</p><a href=\"/\">Go About</a></body></html>".to_string()
        });

    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(html)
        .unwrap())
}

#[tracing::instrument(name = "server_error", skip(state))]
pub async fn serve_500(State(state): State<AppState>) -> Result<impl IntoResponse> {
    tracing::error!("serving 500 page");

    let context = HashMap::new();
    let html = state.template_engine.render_template("templates/500.html", &context)
        .unwrap_or_else(|_| {
            // Fallback 500 page
            "<!DOCTYPE html><html><head><title>500 - Server Error</title></head><body><h1>500 - Server Error</h1><p>Something went wrong on our end.</p><a href=\"/\">Go About</a></body></html>".to_string()
        });

    Ok(Response::builder()
        .status(500)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(html)
        .unwrap())
}
