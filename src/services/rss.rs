use crate::{config::Config, models::BlogPost};
use rss::{Category, ChannelBuilder, GuidBuilder, ItemBuilder};

pub struct RssService;

impl RssService {
    pub fn generate_feed(config: &Config, posts: &[BlogPost]) -> String {
        let mut items = Vec::new();

        for post in posts.iter().take(20) {
            // Limit to most recent 20 posts
            let guid = GuidBuilder::default()
                .value(format!("https://{}/blog/{}", config.site.domain, post.slug))
                .permalink(true)
                .build();

            let categories: Vec<Category> = post
                .tags
                .iter()
                .map(|tag| Category {
                    name: tag.clone(),
                    domain: None,
                })
                .collect();

            let item = ItemBuilder::default()
                .title(Some(post.title.clone()))
                .link(Some(format!(
                    "https://{}/blog/{}",
                    config.site.domain, post.slug
                )))
                .description(Some(post.description.clone()))
                .pub_date(Some(post.iso_date()))
                .guid(Some(guid))
                .author(Some(format!(
                    "{} ({})",
                    config.site.author, "juan@jrada.dev"
                )))
                .categories(categories)
                .build();
            items.push(item);
        }

        let channel = ChannelBuilder::default()
            .title(&config.site.title)
            .link(format!("https://{}", config.site.domain))
            .description(&config.site.description)
            .language(Some(config.site.language.clone()))
            .managing_editor(Some(format!(
                "{} ({})",
                config.site.author, "juan@jrada.dev"
            )))
            .webmaster(Some(format!(
                "{} ({})",
                config.site.author, "juan@jrada.dev"
            )))
            .copyright(Some(format!(
                "© 2025 {}. All rights reserved.",
                config.site.author
            )))
            .generator(Some("Rust Blog Engine v1.0".to_string()))
            .docs(Some(
                "https://www.rssboard.org/rss-specification".to_string(),
            ))
            .ttl(Some("1440".to_string())) // 24 hours
            .items(items)
            .build();

        channel.to_string()
    }
}
