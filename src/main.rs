mod cli;
mod extractor;
mod models;
mod output;

use scraper::Html;
use std::{fs::File, io::Write, path::Path, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = cli::Config::parse();
    println!("Fetching content from: {}", config.url);

    // Add file extension based on format
    let output_path = match config.output_format.as_str() {
        "json" => format!("{}.json", config.output_file),
        "html" => format!("{}.html", config.output_file),
        _ => format!("{}.txt", config.output_file),
    };

    // If we set a delay, wait before making the request
    if config.delay_ms > 0 {
        println!(
            "Waiting for {} milliseconds before request...",
            config.delay_ms
        );
        tokio::time::sleep(Duration::from_millis(config.delay_ms)).await;
    }

    // Get the HTML content with a timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(&config.url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let result = extractor::extract(&config.url, &document)?;

        // Save the results based on the specific format
        match config.output_format.as_str() {
            "json" => output::json::save(&result, &output_path)?,
            "html" => output::html::save(&result, &output_path)?,
            _ => output::text::save(&result, &output_path)?,
        }

        println!("\nResults saved to: {}", output_path);
    } else {
        let error_msg = format!("Failed to retrieve the webpage: {}", response.status());
        eprintln!("{}", error_msg);

        // Save error message to file
        let mut output_file = File::create(Path::new(&output_path))?;
        writeln!(output_file, "{}", error_msg)?;
    }

    Ok(())
}
