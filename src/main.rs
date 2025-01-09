use axum::{
    response::{Html, Redirect},
    routing::get,
    Router,
};
use pulldown_cmark::{html, Parser};
use serde::Deserialize;
use std::fs;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    domain: String,
    posts: Vec<BlogPost>,
}

#[derive(Deserialize, Clone)]
struct BlogPost {
    title: String,
    date: String,
    description: String,
    content: String,
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Load and parse blog posts from assets/posts/*.md
    let posts = load_blog_posts().unwrap_or_default();
    let state = AppState {
        domain: "localhost:8000".to_string(),
        posts,
    };

    let router = Router::new()
        .route("/", get(serve_index))
        .route("/blog", get(serve_blog))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    Ok(router.into())
}

async fn serve_index() -> impl axum::response::IntoResponse {
    // Read and serve index.html
    let content = fs::read_to_string("assets/index.html").unwrap_or_else(|_| "404".to_string());
    Html(content)
}

async fn serve_blog(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    // Read blog.html template
    let template = fs::read_to_string("assets/blog.html").unwrap_or_else(|_| "404".to_string());

    // Insert blog posts into template
    let mut blog_content = String::new();
    for post in state.posts {
        blog_content.push_str(&format!(
            r#"<article class="blog-post">
                <h2>{}</h2>
                <div class="blog-post-meta">Published on {}</div>
                <div class="blog-post-description">{}</div>
            </article>"#,
            post.title, post.date, post.description
        ));
    }

    // Replace placeholder in template with blog content
    let html = template.replace("<!-- BLOG_POSTS -->", &blog_content);
    Html(html)
}

fn load_blog_posts() -> std::io::Result<Vec<BlogPost>> {
    let mut posts = Vec::new();
    let posts_dir = fs::read_dir("assets/posts")?;

    for entry in posts_dir {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(entry.path())?;
            // Parse frontmatter and markdown
            if let Some(post) = parse_blog_post(&content) {
                posts.push(post);
            }
        }
    }

    // Sort posts by date, newest first
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(posts)
}

fn parse_blog_post(content: &str) -> Option<BlogPost> {
    // Simple frontmatter parser - you may want to use a proper YAML parser
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

    // Parse markdown content
    let markdown = parts[2];
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Some(BlogPost {
        title: title.to_string(),
        date: date.to_string(),
        description: description.to_string(),
        content: html_output,
    })
}
