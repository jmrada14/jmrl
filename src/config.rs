use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub site: SiteConfig,
    pub server: ServerConfig,
    pub cache: CacheConfig,
    pub social: SocialConfig,
    pub analytics: AnalyticsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub domain: String,
    pub description: String,
    pub author: String,
    pub language: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub static_files_max_age: u64,
    pub html_max_age: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SocialConfig {
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsConfig {
    pub google_analytics_id: Option<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Path::new("config.toml");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Config file not found at config.toml"));
        }

        let config_str = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn cache_control_static(&self) -> String {
        format!("public, max-age={}, immutable", self.cache.static_files_max_age)
    }

    pub fn cache_control_html(&self) -> String {
        format!("public, max-age={}", self.cache.html_max_age)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            site: SiteConfig {
                title: "Juan M. Rada León".to_string(),
                domain: "localhost:8080".to_string(),
                description: "Personal website and blog".to_string(),
                author: "Juan M. Rada León".to_string(),
                language: "en".to_string(),
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            cache: CacheConfig {
                static_files_max_age: 31536000,
                html_max_age: 3600,
            },
            social: SocialConfig {
                linkedin: None,
                github: None,
                email: None,
            },
            analytics: AnalyticsConfig {
                google_analytics_id: None,
            },
        }
    }
}
