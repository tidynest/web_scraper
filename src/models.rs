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
