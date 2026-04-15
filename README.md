# Web Scraper

A flexible web scraper built in Rust that can extract and save various elements from websites.

## Features

- Interactive URL prompt when no URL provided via arguments
- Extracts page title, links, headers (h1–h6), and meta tags (name, OpenGraph, http-equiv)
- Saves output in multiple formats (text, JSON, HTML)
- Command-line arguments for easy customization
- Delay option to respect rate limits
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
# Add a delay before making the request (in milliseconds)
web_scraper --url <url> --delay 2000
```

### Full Example

```bash
web_scraper --url <url> --format html --output my_results --delay 1000
```

## Output Files

The scraper will create one of these files depending on the format:

- `scraping_results.txt` (default)
- `scraping_results.json` (with `--format json`)
- `scraping_results.html` (with `--format html`)

You can change the base name with the `--output` option.

## Note

This is a basic web scraper for educational purposes. Be respectful when scraping websites:

- Check the website's robots.txt file for scraping permissions
- Use reasonable delays between requests
- Don't overload servers with too many requests

## License

MIT