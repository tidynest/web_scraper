use crate::models::*;
use scraper::{Html, Selector};
use std::collections::HashSet;

pub fn extract(url: &str, document: &Html) -> Result<ScrapingResult, Box<dyn std::error::Error>> {
    // Track what we've found
    let mut result = ScrapingResult {
        url: url.to_string(),
        title: None,
        links: Vec::new(),
        headers: Vec::new(),
        meta_tags: Vec::new(),
    };

    // Get the title
    let title_selector = Selector::parse("title")?;
    if let Some(element) = document.select(&title_selector).next() {
        let title = element.text().collect::<String>();
        println!("Title: {}", title);
        result.title = Some(title);
    } else {
        println!("No title found");
    }

    // Get all links
    println!("\nLinks found:");
    let links_selector = Selector::parse("a")?;
    let mut unique_links = HashSet::new();

    for (i, link) in document.select(&links_selector).enumerate() {
        if let Some(href) = link.value().attr("href") {
            let link_text = link.text().collect::<String>();
            let display_text = if link_text.trim().is_empty() {
                "[No text]".to_string()
            } else {
                link_text.trim().to_string()
            };

            let output_line = format!("{}. {} -> {}", i + 1, display_text, href);

            // Only print if we haven't seen this link before
            if unique_links.insert((display_text.clone(), href.to_string())) {
                println!("{}", output_line);

                // Add to out results
                result.links.push(Link {
                    text: display_text,
                    url: href.to_string(),
                });
            }
        }
    }

    // Get all headers (h1, h2, h3, etc.)
    println!("\nHeaders found:");
    let mut header_count = 0;

    for level in 1..=6 {
        // Use static strings instead of format!
        let selector_str = match level {
            1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            6 => "h6",
            _ => continue, // Skip any other levels
        };

        let header_selector = Selector::parse(selector_str)?;
        for header in document.select(&header_selector) {
            let header_text = header.text().collect::<String>().trim().to_string();
            header_count += 1;
            println!("H{}: {}", level, header_text);

            result.headers.push(Header {
                level,
                text: header_text,
            });
        }
    }

    if header_count == 0 {
        println!("No headers found");
    }

    // Get the meta elements
    println!("\nMeta tags found:");
    let meta_selector = Selector::parse("meta")?;
    for element in document.select(&meta_selector) {
        let name = element
            .value()
            .attr("name")
            .or(element.value().attr("property"))
            .or(element.value().attr("http-equiv"));

        if let (Some(name), Some(content)) = (name, element.value().attr("content")) {
            println!("{}: {}", name, content);

            // Add to out results
            result.meta_tags.push(MetaTag {
                name: name.to_string(),
                content: content.to_string(),
            });
        }
    }

    Ok(result)
}
