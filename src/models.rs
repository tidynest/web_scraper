use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScrapingResult {
    pub url: String,
    pub title: Option<String>,
    pub links: Vec<Link>,
    pub headers: Vec<Header>,
    pub meta_tags: Vec<MetaTag>,
    pub images: Vec<Image>,
    pub metrics: Metrics,
}
impl ScrapingResult {
    /// Keep only entries whose text or URL contains `keyword` (case-insensitive).
    pub fn apply_filter(&mut self, keyword: &str) {
        let kw = keyword.to_lowercase();
        let hit = |s: &str| s.to_lowercase().contains(&kw);
        self.links.retain(|link| hit(&link.text) || hit(&link.url));
        self.headers.retain(|header| hit(&header.text));
        self.meta_tags
            .retain(|meta_tag| hit(&meta_tag.name) || hit(&meta_tag.content));
        self.images
            .retain(|image| hit(&image.url) || hit(&image.alt));
    }
}

#[derive(Serialize, Deserialize)]
pub struct Link {
    pub text: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Header {
    pub level: u8,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct MetaTag {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub alt: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Metrics {
    pub fetch_time_ms: u128,
    pub parse_time_ms: u128,
    pub page_size_bytes: usize,
}
impl Metrics {
    pub fn size_display(&self) -> String {
        if self.page_size_bytes >= 1024 {
            format!("{:.1} KiB", self.page_size_bytes as f64 / 1024.0)
        } else {
            format!("{} bytes", self.page_size_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ScrapingResult {
        ScrapingResult {
            url: "http://example.com".into(),
            title: None,
            links: vec![
                Link {
                    text: "Rust docs".into(),
                    url: "https://doc.rust-lang.org".into(),
                },
                Link {
                    text: "News".into(),
                    url: "https://example.com/news".into(),
                },
            ],
            headers: vec![Header {
                level: 1,
                text: "Why Rust".into(),
            }],
            meta_tags: Vec::new(),
            images: Vec::new(),
            metrics: Metrics::default(),
        }
    }

    #[test]
    fn filter_is_case_insensitive_and_checks_text_and_url() {
        let mut result = sample();
        result.apply_filter("RUST");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.headers.len(), 1);
        assert_eq!(result.links[0].text, "Rust docs");
    }
}
