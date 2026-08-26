use crate::models::ScrapingResult;
use std::{fs::File, io::Write, path::Path};

// Order matters: Escape & first or you double-escape the others
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn save(result: &ScrapingResult, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(Path::new(output_path))?;
    writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(file, "<scraping_result url=\"{}\">", esc(&result.url))?;
    if let Some(title) = &result.title {
        writeln!(file, "  <title>{}</title>", esc(title))?;
    }
    writeln!(file, "  <links>")?;
    for link in &result.links {
        writeln!(
            file,
            r#"    <link url="{}">{}</link>"#,
            esc(&link.url),
            esc(&link.text)
        )?;
    }
    writeln!(file, "  </links>")?;
    writeln!(file, "  <headers>")?;
    for header in &result.headers {
        writeln!(
            file,
            r#"    <header level="{}">{}</header>"#,
            header.level,
            esc(&header.text)
        )?;
    }
    writeln!(file, "  </headers>")?;
    writeln!(file, "  <meta_tags>")?;
    for meta in &result.meta_tags {
        writeln!(
            file,
            r#"    <meta name="{}">{}</meta>"#,
            esc(&meta.name),
            esc(&meta.content)
        )?;
    }
    writeln!(file, "  </meta_tags>")?;
    writeln!(file, "  <images>")?;
    for image in &result.images {
        writeln!(
            file,
            r#"    <image alt="{}">{}</image>"#,
            esc(&image.alt),
            esc(&image.url)
        )?;
    }
    writeln!(file, "  </images>")?;
    let m = &result.metrics;
    writeln!(
        file,
        "  <metrics fetch_time_ms=\"{}\" parse_time_ms=\"{}\" page_size_bytes=\"{}\"/>",
        m.fetch_time_ms, m.parse_time_ms, m.page_size_bytes
    )?;
    writeln!(file, "</scraping_result>")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::esc;

    #[test]
    fn escapes_xml_special_chars() {
        assert_eq!(esc(r#"a & <b> "c""#), "a &amp; &lt;b&gt; &quot;c&quot;");
    }
}
