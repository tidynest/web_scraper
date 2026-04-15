use std::env;
use std::io::{self, Write};

pub struct Config {
    pub url: String,
    pub output_format: String,
    pub output_file: String,
    pub delay_ms: u64,
}

impl Config {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut url = String::new();
        let mut output_format = String::from("text");
        let mut output_file = String::from("scraping_results");
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
                    output_format = args[i + 1].clone();
                    i += 1;
                }
                "--output" if i + 1 < args.len() => {
                    output_file = args[i + 1].clone();
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
            url = Self::prompt_for_url();
        }

        Self { url, output_format, output_file, delay_ms }
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
}

