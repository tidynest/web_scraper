use scraper::{Html, Selector};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

// Import for JSON output
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

#[derive(Serialize, Deserialize)]
struct ScrapingResult {
    url: String,
    title: Option<String>,
    links: Vec<Link>,
    headers: Vec<Header>,
    meta_tags: Vec<MetaTag>,
}

#[derive(Serialize, Deserialize)]
struct Link {
    text: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Header {
    level: u8,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct MetaTag {
    name: String,
    content: String,
}

fn prompt_for_url() -> String {
    print!("Enter target URL: ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let url = input.trim().to_string();
    if url.is_empty() || !url.starts_with("http") {
        eprintln!("Invalid URL. Must start with http:// or https://");
        std::process::exit(1);
    }
    url
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Set defaults
    let mut url = String::new();
    let mut output_format = "text";
    let mut output_file = "scraping_results";
    let mut delay_ms: u64 = 0;

    // Process command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" if i + 1 < args.len() => {
                url = args[i + 1].clone();
                i += 1;
            }
            "--format" if i + 1 < args.len() => {
                output_format = &args[i + 1];
                i += 1;
            }
            "--output" if i + 1 < args.len() => {
                output_file = &args[i + 1];
                i += 1;
            }
            "--delay" if i + 1 < args.len() => {
                delay_ms = args[i + 1].parse().unwrap_or(0);
                i += 1;
            }
            _ if i == 1 && !args[i].starts_with("--") => url = args[i].clone(),
            _ => {}
        }
        i += 1;
    }

    if url.is_empty() {
        url = prompt_for_url();
    }

    println!("Fetching content from: {}", url);

    // Add file extension based on format
    let output_path = match output_format {
        "json" => format!("{}.json", output_file),
        "html" => format!("{}.html", output_file),
        _ => format!("{}.txt", output_file),
    };

    // If we set a delay, wait before making the request
    if delay_ms > 0 {
        println!("Waiting for {} milliseconds before request...", delay_ms);
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // Get the HTML content with a timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let document = Html::parse_document(&body);

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

            let header_selector = Selector::parse(&selector_str)?;
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
            let name = element.value().attr("name")
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

        // Save the results based on the specific format
        match output_format {
            "json" => save_as_json(&result, &output_path)?,
            "html" => save_as_html(&result, &output_path)?,
            _ => save_as_text(&result, &output_path)?,
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

fn save_as_text(
    result: &ScrapingResult,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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

fn save_as_json(
    result: &ScrapingResult,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(Path::new(output_path))?;
    let json = to_string_pretty(result)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn save_as_html(
    result: &ScrapingResult,
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
    writeln!(file, "  <h2>Meta Tags Found ({})</h2>", result.meta_tags.len())?;
    if result.meta_tags.is_empty() {
        writeln!(file, "  <p>No Meta Tags Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"meta-tags\">")?;
        for meta_tag in &result.meta_tags {
            writeln!(file, "    <li><strong>{}</strong>: {}</li>", meta_tag.name, meta_tag.content)?;
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
