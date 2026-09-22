mod cli;
mod crawler;
mod extractor;
mod models;
mod output;
mod screenshot;

use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = cli::Config::parse();
    println!("Fetching content from: {}", config.url);

    // Add file extension based on format
    let ext = match config.output_format.as_str() {
        "json" | "html" | "csv" | "xml" => config.output_format.as_str(),
        _ => "txt",
    };
    let output_path = format!("{}.{}", config.output_file, ext);

    // Get the HTML content with a timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut results = crawler::crawl(
        &client,
        &config.url,
        config.depth,
        config.delay_ms,
        config.concurrency,
    )
    .await?;

    if let Some(keyword) = &config.filter {
        for result in &mut results {
            result.apply_filter(keyword);
        }
    }

    if config.screenshot
        && let Err(e) = screenshot::capture(&mut results, &config.output_file).await
    {
        eprintln!("Screenshots skipped: {}", e);
    }

    if results.is_empty() {
        eprintln!("No pages could be retrieved.");
        std::process::exit(1);
    }

    match config.output_format.as_str() {
        "json" => output::json::save(&results, &output_path)?,
        "html" => output::html::save(&results, &output_path)?,
        "csv" => output::csv::save(&results, &output_path)?,
        "xml" => output::xml::save(&results, &output_path)?,
        _ => output::text::save(&results, &output_path)?,
    }

    println!("\n{} page(s) saved to: {}", results.len(), output_path);
    Ok(())
}
