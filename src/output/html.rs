use crate::models::ScrapingResult;
use std::{fs::File, io::Write, path::Path};

pub fn save(result: &ScrapingResult, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(Path::new(output_path))?;

    // Write HTML header
    writeln!(file, "<!DOCTYPE html>")?;
    writeln!(file, "<html lang=\"en\">")?;
    writeln!(file, "<head>")?;
    writeln!(file, "  <meta charset=\"UTF-8\">")?;
    writeln!(
        file,
        "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
    )?;
    writeln!(file, "  <title>Scraping Results for {}</title>", result.url)?;
    writeln!(file, "  <style>")?;
    writeln!(
        file,
        "    body {{ font-family: Arial, sans-serif; margin: 20px; }}"
    )?;
    writeln!(file, "    h1 {{ color: #2c3e50; }}")?;
    writeln!(file, "    h2 {{ color: #3498db; margin-top: 30px; }}")?;
    writeln!(file, "    .url {{ color: #7f8c8f; font-style: italic; }}")?;
    writeln!(file, "    .links {{ margin-top: 20px; }}")?;
    writeln!(file, "    .links li {{ margin-bottom: 5px; }}")?;
    writeln!(file, "    .headers {{ margin-top: 20px; }}")?;
    writeln!(file, "    .headers li {{ margin-bottom: 5px; }}")?;
    writeln!(
        file,
        "    .headers-tag {{ color: #e74c3c; font-weight: bold; }}"
    )?;
    writeln!(file, "  </style>")?;
    writeln!(file, "</head>")?;
    writeln!(file, "<body>")?;

    // Page title
    writeln!(file, "  <h1>Web Scraping Results</h1>")?;
    writeln!(file, "  <p class=\"url\">Source: {}</p>", result.url)?;

    // Page info
    if let Some(title) = &result.title {
        writeln!(file, "  <h2>Page Title</h2>")?;
        writeln!(file, "  <p>{}</p>", title)?;
    }

    // Links section
    writeln!(file, "  <h2>Links found ({})</h2>", result.links.len())?;
    if result.links.is_empty() {
        writeln!(file, "  <p>No Links Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"links\">")?;
        for link in &result.links {
            writeln!(
                file,
                "    <li><a href=\"{}\">{}</a></li>",
                link.url, link.text
            )?;
        }
        writeln!(file, "  </ul>")?;
    }

    // Headers section
    writeln!(file, "  <h2>Headers Found ({})</h2>", result.headers.len())?;
    if result.headers.is_empty() {
        writeln!(file, "  <p>No Headers Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"headers\">")?;
        for header in &result.headers {
            writeln!(
                file,
                "    <li><span class=\"header-tag\">H{}</span>: {}</li>",
                header.level, header.text
            )?;
        }
        writeln!(file, "  </ul>")?;
    }

    // Meta tags section
    writeln!(
        file,
        "  <h2>Meta Tags Found ({})</h2>",
        result.meta_tags.len()
    )?;
    if result.meta_tags.is_empty() {
        writeln!(file, "  <p>No Meta Tags Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"meta-tags\">")?;
        for meta_tag in &result.meta_tags {
            writeln!(
                file,
                "    <li><strong>{}</strong>: {}</li>",
                meta_tag.name, meta_tag.content
            )?;
        }
        writeln!(file, "  </ul>")?;
    }

    // Close HTML tags
    writeln!(file, "  <hr>")?;
    writeln!(
        file,
        "  <p><small>Generated on: {}</small></p>",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )?;
    writeln!(file, "</body>")?;
    writeln!(file, "</html>")?;

    Ok(())
}
