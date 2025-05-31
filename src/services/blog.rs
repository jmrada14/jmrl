use crate::{error::Result, models::BlogPost};
use pulldown_cmark::{html, Parser};
use regex::Regex;
use std::fs;

pub struct BlogService;

impl BlogService {
    pub fn load_posts() -> Result<Vec<BlogPost>> {
        let mut posts = Vec::new();
        let posts_dir = fs::read_dir("content/posts")
            .or_else(|_| fs::read_dir("assets/posts"))?; // Fallback to old location

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

                if let Some(post) = Self::parse_blog_post(&content, &filename)? {
                    posts.push(post);
                }
            }
        }

        posts.sort();
        Ok(posts)
    }

    fn parse_blog_post(content: &str, filename: &str) -> Result<Option<BlogPost>> {
        let parts: Vec<&str> = content.split("---").collect();
        if parts.len() < 3 {
            tracing::warn!("Invalid blog post format in file: {}", filename);
            return Ok(None);
        }

        // Parse frontmatter
        let frontmatter = parts[1];
        let title = Self::extract_frontmatter_field(frontmatter, "title")?;
        let date = Self::extract_frontmatter_field(frontmatter, "date")?;
        let description = Self::extract_frontmatter_field(frontmatter, "description")?;
        
        // Extract tags (optional)
        let tags = Self::extract_frontmatter_list(frontmatter, "tags")
            .unwrap_or_default();

        // Parse markdown content
        let markdown = parts[2..].join("---"); // Rejoin in case there are more --- in content
        let parser = Parser::new(&markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Remove .md extension for the path
        let path = filename.strip_suffix(".md").unwrap_or(filename).to_string();

        Ok(Some(BlogPost::new(
            title,
            date,
            description,
            html_output,
            path,
            tags,
        )))
    }

    fn extract_frontmatter_field(frontmatter: &str, field: &str) -> Result<String> {
        let pattern = format!(r"{}:\s*(.+)", field);
        let re = Regex::new(&pattern).unwrap();
        
        re.captures(frontmatter)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().trim_matches('"').to_string())
            .ok_or_else(|| crate::error::AppError::BlogParsing(
                format!("Missing {} field in frontmatter", field)
            ))
    }

    fn extract_frontmatter_list(frontmatter: &str, field: &str) -> Result<Vec<String>> {
        let pattern = format!(r"{}:\s*\[(.*?)\]", field);
        let re = Regex::new(&pattern).unwrap();
        
        if let Some(caps) = re.captures(frontmatter) {
            if let Some(list_content) = caps.get(1) {
                let tags: Vec<String> = list_content
                    .as_str()
                    .split(',')
                    .map(|tag| tag.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect();
                return Ok(tags);
            }
        }
        Ok(Vec::new())
    }

    pub fn get_post_by_slug<'a>(posts: &'a [BlogPost], slug: &str) -> Option<&'a BlogPost> {
        posts.iter().find(|post| post.slug == slug || post.path == slug)
    }
}
