use crate::{config::Config, models::BlogPost};

pub struct SitemapService;

impl SitemapService {
    pub fn generate_sitemap(config: &Config, posts: &[BlogPost]) -> String {
        let mut sitemap = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:news="http://www.google.com/schemas/sitemap-news/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml"
        xmlns:mobile="http://www.google.com/schemas/sitemap-mobile/1.0"
        xmlns:image="http://www.google.com/schemas/sitemap-image/1.1"
        xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
"#,
        );

        // Add homepage (about page)
        sitemap.push_str(&format!(
            r#"  <url>
    <loc>https://{}</loc>
    <changefreq>weekly</changefreq>
    <priority>1.0</priority>
  </url>
"#,
            config.site.domain
        ));

        // Add blog index
        sitemap.push_str(&format!(
            r#"  <url>
    <loc>https://{}/blog</loc>
    <changefreq>weekly</changefreq>
    <priority>0.9</priority>
  </url>
"#,
            config.site.domain
        ));

        // Add RSS feed
        sitemap.push_str(&format!(
            r#"  <url>
    <loc>https://{}/feed.xml</loc>
    <changefreq>daily</changefreq>
    <priority>0.5</priority>
  </url>
"#,
            config.site.domain
        ));

        // Add blog posts with better priorities based on recency
        for (index, post) in posts.iter().enumerate() {
            // Newer posts get higher priority
            let priority = if index < 5 {
                0.8 // Recent posts
            } else if index < 10 {
                0.7 // Semi-recent posts
            } else {
                0.6 // Older posts
            };

            // Extract the date part from ISO date safely
            let iso_date = post.iso_date();
            let post_date = iso_date.split('T').next().unwrap_or(&post.date);

            sitemap.push_str(&format!(
                r#"  <url>
    <loc>https://{}/blog/{}</loc>
    <lastmod>{}</lastmod>
    <changefreq>monthly</changefreq>
    <priority>{:.1}</priority>
  </url>
"#,
                config.site.domain, post.slug, post_date, priority
            ));
        }

        sitemap.push_str("</urlset>");
        sitemap
    }
}
