use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub title: String,
    pub date: String,
    pub date_parsed: Option<DateTime<Utc>>,
    pub description: String,
    pub content: String,
    pub path: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub reading_time: u32,
    pub excerpt: String,
}

impl BlogPost {
    pub fn new(
        title: String,
        date: String,
        description: String,
        content: String,
        path: String,
        tags: Vec<String>,
    ) -> Self {
        let date_parsed = chrono::DateTime::parse_from_str(&date, "%Y-%m-%d")
            .ok()
            .map(|dt| dt.with_timezone(&Utc));

        let slug = Self::generate_slug(&title);
        let reading_time = Self::calculate_reading_time(&content);
        let excerpt = Self::generate_excerpt(&content);

        Self {
            title,
            date,
            date_parsed,
            description,
            content,
            path,
            slug,
            tags,
            reading_time,
            excerpt,
        }
    }

    fn generate_slug(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    fn calculate_reading_time(content: &str) -> u32 {
        let word_count = content.split_whitespace().count();
        ((word_count as f32 / 200.0).ceil() as u32).max(1) // Assume 200 WPM reading speed
    }

    fn generate_excerpt(content: &str) -> String {
        // Strip HTML tags first to get plain text
        let text_content = content
            .replace("<h2>", "")
            .replace("</h2>", " ")
            .replace("<h3>", "")
            .replace("</h3>", " ")
            .replace("<h4>", "")
            .replace("</h4>", " ")
            .replace("<p>", "")
            .replace("</p>", " ")
            .replace("<li>", "")
            .replace("</li>", " ")
            .replace("<ul>", "")
            .replace("</ul>", " ")
            .replace("<ol>", "")
            .replace("</ol>", " ")
            .replace("<br>", " ")
            .replace("<br />", " ")
            .replace("<em>", "")
            .replace("</em>", "")
            .replace("<strong>", "")
            .replace("</strong>", "")
            .replace("<blockquote>", "")
            .replace("</blockquote>", " ");

        // Remove code blocks entirely to avoid broken code snippets
        let text_without_code = regex::Regex::new(r"<pre><code.*?</code></pre>")
            .unwrap()
            .replace_all(&text_content, " ");

        // Clean up multiple spaces and trim
        let cleaned = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&text_without_code, " ")
            .trim()
            .to_string();

        let words: Vec<&str> = cleaned.split_whitespace().collect();
        if words.len() <= 30 {
            cleaned
        } else {
            words[..30].join(" ") + "..."
        }
    }

    pub fn formatted_date(&self) -> String {
        if let Some(parsed_date) = self.date_parsed {
            parsed_date.format("%B %d, %Y").to_string()
        } else {
            self.date.clone()
        }
    }

    pub fn iso_date(&self) -> String {
        if let Some(parsed_date) = self.date_parsed {
            parsed_date.to_rfc3339()
        } else {
            self.date.clone()
        }
    }
}

impl PartialEq for BlogPost {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for BlogPost {}

impl PartialOrd for BlogPost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlogPost {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by date (newest first), then by title
        match (self.date_parsed, other.date_parsed) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => other.date.cmp(&self.date),
        }
    }
}
