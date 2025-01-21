use axum::http::{header, HeaderValue};
use axum::{extract::Path, response::Response, routing::get, Router};
use pulldown_cmark::{html, Parser};
use serde::Deserialize;
use std::fs;
use tower::{Layer, ServiceBuilder};
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Clone, Debug)]
struct AppState {
    #[allow(dead_code)]
    domain: String,
    posts: Vec<BlogPost>,
}

#[derive(Deserialize, Clone, Debug)]
struct BlogPost {
    title: String,
    date: String,
    description: String,
    content: String,
    path: String,
    metatags: String,
}
fn cache_headers() -> [(&'static str, &'static str); 2] {
    [
        ("Cache-Control", "public, max-age=31536000, immutable"),
        ("X-Content-Type-Options", "nosniff"),
    ]
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    tracing::info!("tracing is initialized");

    let posts = load_blog_posts().unwrap_or_default();
    let state = AppState {
        domain: "localhost:8000".to_string(),
        posts,
    };

    let router: Router = Router::new()
        .route("/", get(serve_index))
        .route("/blog", get(serve_blog))
        .nest_service(
            "/assets",
            tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            )
            .layer(ServeDir::new("assets")),
        )
        .route("/blog/:post", get(serve_blog_post))
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[tracing::instrument(name = "index")]
async fn serve_index() -> impl axum::response::IntoResponse {
    tracing::info!("serving index",);
    let content = fs::read_to_string("assets/index.html").unwrap_or_else(|_| "404".to_string());
    let mut response = Response::builder();

    for (name, value) in cache_headers() {
        response = response.header(name, value);
    }

    response
        .header("Content-Type", "text/html; charset=utf-8")
        .body(content)
        .unwrap()
}

#[tracing::instrument(name = "blog")]
async fn serve_blog(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    tracing::info!("serving blog page");
    let template = fs::read_to_string("assets/blog.html").unwrap_or_else(|e| {
        tracing::error!("failed to read blog template: {}", e);
        "404".to_string()
    });

    let mut blog_content = String::new();

    for post in &state.posts {
        blog_content.push_str(&format!(
            r#"<article class="blog-post">
                <a href="/blog/{}">{}</a>
                <div class="blog-post-meta">Published on {}</div>
                <div class="blog-post-description">{}</div>
            </article>"#,
            post.path, post.title, post.date, post.description
        ));
    }

    let html = template.replace("<!-- BLOG_POSTS -->", &blog_content);
    let mut response = Response::builder();

    for (name, value) in cache_headers() {
        response = response.header(name, value);
    }

    response
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html)
        .unwrap()
}
fn load_blog_posts() -> std::io::Result<Vec<BlogPost>> {
    let mut posts = Vec::new();
    let posts_dir = fs::read_dir("assets/posts")?;

    for entry in posts_dir {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(entry.path())?;
            let filename = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if let Some(mut post) = parse_blog_post(&content, &filename) {
                post.path = filename; // Store filename as path
                posts.push(post);
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(posts)
}

fn parse_blog_post(content: &str, filename: &str) -> Option<BlogPost> {
    let parts: Vec<&str> = content.split("---").collect();
    if parts.len() < 3 {
        return None;
    }

    // Parse frontmatter
    let frontmatter = parts[1];
    let title = frontmatter
        .lines()
        .find(|l| l.starts_with("title:"))?
        .split("title:")
        .nth(1)?
        .trim();
    let date = frontmatter
        .lines()
        .find(|l| l.starts_with("date:"))?
        .split("date:")
        .nth(1)?
        .trim();
    let description = frontmatter
        .lines()
        .find(|l| l.starts_with("description:"))?
        .split("description:")
        .nth(1)?
        .trim();
    let metatags = frontmatter
        .lines()
        .find(|l| l.starts_with("metatags:"))?
        .split("metatags:")
        .nth(1)?
        .trim();
    // Parse markdown content
    let markdown = parts[2];
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Remove .md extension for the path
    let path = filename.strip_suffix(".md").unwrap_or(filename).to_string();

    Some(BlogPost {
        title: title.to_string(),
        date: date.to_string(),
        description: description.to_string(),
        content: html_output,
        path,
        metatags: metatags.to_string(),
    })
}

#[tracing::instrument(name = "blog_post", fields(post_path = %post_path))]
async fn serve_blog_post(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(post_path): Path<String>,
) -> impl axum::response::IntoResponse {
    tracing::info!("serving blog post: {}", post_path);

    if let Some(post) = state.posts.iter().find(|p| p.path == post_path) {
        tracing::debug!("found post: {}", post.title);
        let template = fs::read_to_string("assets/post.html").unwrap_or_else(|e| {
            tracing::error!("failed to read post template: {}", e);
            "404".to_string()
        });
        let blog_content = format!(
            r#"<article class="blog-post">
                <div class="blog-post-content">{}</div>
            </article>"#,
            post.content
        );

        let html = template
            .replace("<!-- BLOG_TITLE -->", &post.title)
            .replace("<!-- BLOG_DATE -->", &post.date)
            .replace("<!-- BLOG_POSTS -->", &blog_content)
            .replace(
                "<!-- META_TAGS -->",
                &format!(
                    r#"<meta name="keywords" content="{}">
               <meta name="description" content="{}">"#,
                    post.metatags, post.description
                ),
            );

        let mut response = Response::builder();

        for (name, value) in cache_headers() {
            response = response.header(name, value);
        }

        response
            .header("Content-Type", "text/html; charset=utf-8")
            .body(html)
            .unwrap()
    } else {
        tracing::warn!("post not found: {}", post_path);
        let mut response = Response::builder();

        for (name, value) in cache_headers() {
            response = response.header(name, value);
        }

        response
            .header("Content-Type", "text/html; charset=utf-8")
            .status(404)
            .body("<h1>404 Not Found</h1>".to_string())
            .unwrap()
    }
}
