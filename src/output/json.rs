use crate::models::ScrapingResult;
use serde_json::to_string_pretty;
use std::{fs::File, io::Write, path::Path};

pub fn save(
    results: &[ScrapingResult],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(Path::new(output_path))?;
    let json = to_string_pretty(results)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
