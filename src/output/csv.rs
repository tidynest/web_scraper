use crate::models::ScrapingResult;
use std::{fs::File, io::Write, path::Path};

// RFC 4180: quote every field, double inner quotes; embedded newlines are then legal
fn esc(field: &str) -> String {
    format!("\"{}\"", field.replace('"', "\"\""))
}

pub fn save(result: &ScrapingResult, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(Path::new(output_path))?;
    writeln!(file, "kind,field1,field2")?;
    writeln!(file, "url,{},", esc(&result.url))?;
    if let Some(title) = &result.title {
        writeln!(file, "title,{},", esc(title))?;
    }
    for link in &result.links {
        writeln!(file, "link,{},{}", esc(&link.text), esc(&link.url))?;
    }
    for header in &result.headers {
        writeln!(file, "header,{},{}", header.level, esc(&header.text))?;
    }
    for meta in &result.meta_tags {
        writeln!(file, "meta,{},{}", esc(&meta.name), esc(&meta.content))?;
    }
    for image in &result.images {
        writeln!(file, "image,{},{}", esc(&image.alt), esc(&image.url))?;
    }
    let m = &result.metrics;
    writeln!(file, "metric,fetch_time_ms,{}", m.fetch_time_ms)?;
    writeln!(file, "metric,parse_time_ms,{}", m.parse_time_ms)?;
    writeln!(file, "metric,page_size_bytes,{}", m.page_size_bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::esc;

    #[test]
    fn quotes_and_doubles_inner_quotes() {
        assert_eq!(esc(r#"say "hi", ok"#), r#""say ""hi"", ok""#);
    }
}
