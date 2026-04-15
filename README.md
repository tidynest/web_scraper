# Web Scraper

A flexible web scraper built in Rust that can extract and save various elements from websites.

## Features

- Extracts page title, links, and headers
- Saves output in multiple formats (text, JSON, HTML)
- Command-line arguments for easy customization
- Delay option to respect rate limits
- Timeout handling and error management
- Duplicate link detection

## Installation

Make sure you have Rust and Cargo installed. Then clone this repository and build the project:

```bash
cargo build --release
```

The executable will be available in `target/release/web_scraper`.

## Usage

### Basic Usage

```bash
# Scrape the default example.com site
./web_scraper

# Scrape a specific site
./web_scraper <insert_url_address>

# Or use the --url flag
./web_scraper --url <insert_url_address>
```

### Output Options

```bash
# Save as JSON
./web_scraper --url <insert_url_address> --format json

# Save as HTML
./web_scraper --url <insert_url_address> --format html

# Custom output filename
./web_scraper --url <insert_url_address> --output results
```

### Additional Options

```bash
# Add a delay before making the request (in milliseconds)
./web_scraper --url <insert_url_address> --delay 2000
```

### Full Example

```bash
./web_scraper --url <insert_url_address> --format html --output hts_results --delay 1000
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