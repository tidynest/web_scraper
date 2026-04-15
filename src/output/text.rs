use crate::models::ScrapingResult;
use std::{fs::File, io::Write, path::Path};

pub fn save(result: &ScrapingResult, output_path: &str) -> Result<(), Box<dyn
std::error::Error>> {
    let mut file = File::create(Path::new(output_path))?;

    writeln!(file, "Web Scraping Results for: {}\n", result.url)?;

    if let Some(title) = &result.title {
        writeln!(file, "Title: {}\n", title)?;
    } else {
        writeln!(file, "No title found\n")?;
    }

    writeln!(file, "Links found:")?;
    for (i, link) in result.links.iter().enumerate() {
        writeln!(file, "{}. {} -> {}", i + 1, link.text, link.url)?;
    }

    writeln!(file, "\nHeaders found:")?;
    if result.headers.is_empty() {
        writeln!(file, "No headers found")?;
    } else {
        for header in &result.headers {
            writeln!(file, "H{}: {}", header.level, header.text)?;
        }
    }

    writeln!(file, "\nMeta Tags found:")?;
    if result.meta_tags.is_empty() {
        writeln!(file, "No meta tags found")?;
    } else {
        for meta_tag in &result.meta_tags {
            writeln!(file, "{}: {}", meta_tag.name, meta_tag.content)?;
        }
    }

    Ok(())
}