use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScrapingResult {
    pub url: String,
    pub title: Option<String>,
    pub links: Vec<Link>,
    pub headers: Vec<Header>,
    pub meta_tags: Vec<MetaTag>,
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
