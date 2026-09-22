# Web Scraper

A flexible web scraper built in Rust that can extract and save various elements from websites.

## Features

- Interactive URL prompt when no URL provided via arguments
- Extracts page title, links, headers (h1-h6), meta tags (name, OpenGraph, http-equiv), and image URLs with alt text
- Reports page metrics: content size, fetch time, and parse time
- Saves output in multiple formats (text, JSON, HTML, CSV, XML)
- Command-line arguments for easy customisation
- Crawls same-host links breadth-first to a chosen depth
- Concurrent fetching within each crawl level, capped by `--concurrency`
- Delay option between requests to respect rate limits
- Timeout handling and error management
- Duplicate link detection


## Installation

Make sure you have Rust and Cargo installed.

### Global Install

Installs to `~/.cargo/bin/`, making `web_scraper` available from any directory:

```bash
cargo install --path .
```

To update after code changes, re-run the same command. To uninstall:

```bash
cargo uninstall web_scraper
```

### Local Build

Builds the executable within the project directory:

```bash
cargo build --release
```

The executable will be available at `target/release/web_scraper`.

## Usage

Examples below use `web_scraper` (global install). For local builds, substitute with `./target/release/web_scraper`.

### Basic Usage

```bash
# Run without arguments — prompts for URL interactively
web_scraper

# Pass URL as first argument
web_scraper <url>

# Or use the --url flag
web_scraper --url <url>
```

### Output Options

```bash
# Save as JSON
web_scraper --url <url> --format json

# Save as HTML
web_scraper --url <url> --format html

# Custom output filename
web_scraper --url <url> --output results
```

### Additional Options

```bash
# Add a delay before each request (in milliseconds)
web_scraper --url <url> --delay 2000

# Filter results by keyword
web_scraper --url <url> --filter <keyword>

# Follow same-host links breadth-first, N levels deep (0 = single page, default)
web_scraper --url <url> --depth 1

# Fetch up to N pages at once per crawl level (default 4); pair with --delay to stay polite
web_scraper --url <url> --depth 1 --concurrency 8
```

### Full Example

```bash
web_scraper --url <url> --format html --output my_results --delay 1000 --depth 1
```

## Output Files

The scraper will create one of these files depending on the format:

- `scraping_results.txt` (default)
- `scraping_results.json` (with `--format json`)
- `scraping_results.html` (with `--format html`)
- `scraping_results.csv` (with `--format csv`)
- `scraping_results.xml` (with `--format xml`)

You can change the base name with the `--output` option.

Every format holds all crawled pages in one file. JSON output is an array of page objects, one per page, even at depth 0. Pages that fail to load are skipped with a message on stderr, and the run exits with status 1 only if no page loaded.

## Note

This is a basic web scraper for educational purposes. Be respectful when scraping websites:

- Check the website's robots.txt file for scraping permissions
- Use reasonable delays between requests
- Don't overload servers with too many requests

## License

MIT
