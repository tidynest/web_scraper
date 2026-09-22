use super::{markup_esc as esc, safe_url};
use crate::models::ScrapingResult;
use std::{fs::File, io::Write, path::Path};

pub fn save(
    results: &[ScrapingResult],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
    writeln!(
        file,
        "  <title>Scraping Results for {}</title>",
        esc(results.first().map_or("", |result| result.url.as_str()))
    )?;
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
    writeln!(file, "    .shot {{ border: 1px solid #ccc; }}")?;
    writeln!(file, "  </style>")?;
    writeln!(file, "</head>")?;
    writeln!(file, "<body>")?;

    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            writeln!(file, "  <hr>")?;
        }
        write_page(&mut file, result)?;
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

fn write_page(file: &mut File, result: &ScrapingResult) -> Result<(), Box<dyn std::error::Error>> {
    // Page title
    writeln!(file, "  <h1>Web Scraping Results</h1>")?;
    writeln!(file, "  <p class=\"url\">Source: {}</p>", esc(&result.url))?;

    if let Some(name) = result
        .screenshot
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|name| name.to_str())
    {
        writeln!(
            file,
            "  <a href=\"./{0}\"><img class=\"shot\" src=\"./{0}\" width=\"320\" loading=\"lazy\" alt=\"Page screenshot\"></a>",
            esc(name),
        )?;
    }

    // Page info
    if let Some(title) = &result.title {
        writeln!(file, "  <h2>Page Title</h2>")?;
        writeln!(file, "  <p>{}</p>", esc(title))?;
    }

    // Links section
    writeln!(file, "  <h2>Links found ({})</h2>", result.links.len())?;
    if result.links.is_empty() {
        writeln!(file, "  <p>No Links Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"links\">")?;
        for link in &result.links {
            writeln!(file, "    <li>{}</li>", anchor(&link.url, &link.text),)?;
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
                header.level,
                esc(&header.text),
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
                esc(&meta_tag.name),
                esc(&meta_tag.content),
            )?;
        }
        writeln!(file, "  </ul>")?;
    }

    // Images section
    writeln!(file, "  <h2>Images Found ({})</h2>", result.images.len())?;
    if result.images.is_empty() {
        writeln!(file, "  <p>No Images Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"images\">")?;
        for image in &result.images {
            writeln!(
                file,
                "    <li>{} (alt: {})</li>",
                anchor(&image.url, &image.url),
                esc(&image.alt),
            )?;
        }
        writeln!(file, "  </ul>")?;
    }

    // Metrics section
    let m = &result.metrics;
    writeln!(file, "  <h2>Metrics</h2>")?;
    writeln!(file, "  <ul class=\"metrics\">")?;
    writeln!(
        file,
        "    <li><strong>Page Size:</strong> {}</li>",
        m.size_display()
    )?;
    writeln!(
        file,
        "    <li><strong>Fetch Time:</strong> {} ms</li>",
        m.fetch_time_ms
    )?;
    writeln!(
        file,
        "    <li><strong>Parse Time:</strong> {} ms</li>",
        m.parse_time_ms
    )?;
    writeln!(file, "  </ul>")?;

    Ok(())
}

/// A link for http(s) URLs; any other scheme is shown as text, with the URL on hover.
fn anchor(url: &str, text: &str) -> String {
    match safe_url(url) {
        Some(url) => format!("<a href=\"{}\">{}</a>", esc(url), esc(text)),
        None => format!("<span title=\"{}\">{}</span>", esc(url), esc(text)),
    }
}
