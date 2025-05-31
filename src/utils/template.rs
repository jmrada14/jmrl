use crate::{config::Config, error::Result, models::BlogPost};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TemplateEngine {
    config: Config,
}

impl TemplateEngine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn render_template(
        &self,
        template_path: &str,
        context: &HashMap<String, String>,
    ) -> Result<String> {
        let template_content = std::fs::read_to_string(template_path).map_err(|e| {
            crate::error::AppError::Template(format!(
                "Failed to read template {}: {}",
                template_path, e
            ))
        })?;

        let mut rendered = template_content;
        for (key, value) in context {
            let placeholder = format!("<!-- {} -->", key);
            rendered = rendered.replace(&placeholder, value);
        }

        // Replace config placeholders
        rendered = rendered.replace("<!-- SITE_TITLE -->", &self.config.site.title);
        rendered = rendered.replace("<!-- SITE_DESCRIPTION -->", &self.config.site.description);
        rendered = rendered.replace("<!-- SITE_AUTHOR -->", &self.config.site.author);
        rendered = rendered.replace("<!-- SITE_DOMAIN -->", &self.config.site.domain);

        // Add analytics if configured
        if let Some(ga_id) = &self.config.analytics.google_analytics_id {
            let analytics_script = format!(
                r#"<script async src="https://www.googletagmanager.com/gtag/js?id={}"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){{dataLayer.push(arguments);}}
  gtag('js', new Date());
  gtag('config', '{}');
</script>"#,
                ga_id, ga_id
            );
            rendered = rendered.replace("<!-- ANALYTICS -->", &analytics_script);
        } else {
            rendered = rendered.replace("<!-- ANALYTICS -->", "");
        }

        Ok(rendered)
    }

    pub fn render_blog_list(&self, posts: &[BlogPost]) -> String {
        let mut blog_content = String::new();

        for post in posts {
            let tags_html = if post.tags.is_empty() {
                String::new()
            } else {
                let tag_spans: Vec<String> = post
                    .tags
                    .iter()
                    .map(|tag| format!(r#"<span class="post-tag">{}</span>"#, tag))
                    .collect();
                format!(r#"<div class="post-tags">{}</div>"#, tag_spans.join(""))
            };

            blog_content.push_str(&format!(
                r#"<article class="blog-post-card">
                    <h2><a href="/blog/{}">{}</a></h2>
                    <div class="blog-post-meta">
                        <div class="meta-left">
                            <span class="post-reading-time">{} min read</span>
                            <time datetime="{}">{}</time>
                        </div>
                        <div class="meta-right">
                            <a href="/blog/{}" class="read-more">Read more →</a>
                        </div>
                    </div>
                    {}
                </article>"#,
                post.slug,
                post.title,
                post.reading_time,
                post.iso_date(),
                post.formatted_date(),
                post.slug,
                tags_html
            ));
        }

        blog_content
    }

    pub fn render_post_tags(&self, tags: &[String]) -> String {
        if tags.is_empty() {
            String::new()
        } else {
            let tag_spans: Vec<String> = tags
                .iter()
                .map(|tag| format!(r#"<span class="post-tag">{}</span>"#, tag))
                .collect();
            tag_spans.join("")
        }
    }

    pub fn render_post_tags_plain(&self, tags: &[String]) -> String {
        tags.join(", ")
    }
}
