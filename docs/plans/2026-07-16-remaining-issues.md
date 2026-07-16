# Remaining Issues Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
> **For humans:** work top to bottom; tasks are ordered by dependency. Each task is one GitHub issue, one push.

**Goal:** Close the six remaining issues: images (#1), CSV/XML output (#6), filtering (#4), crawling (#5), concurrency (#8), screenshots (#7).

**Architecture:** Extend the existing pipeline (cli → fetch → extractor → models → output) without reshaping it. Crawling introduces one new module (`crawler.rs`) that owns fetch+extract for N pages; concurrency upgrades its loop. Screenshots are an optional post-processing step in their own module.

**Tech Stack:** Rust 2024, reqwest, scraper, tokio, serde. One new dependency at the very end (`headless_chrome`, issue #7). Everything else is stdlib.

## Global Constraints

- Push gate (`rust-sec-ci`) runs fmt/clippy/audit/deny/check/test on every push — run `cargo fmt && cargo clippy --all-targets` before each commit.
- cargo-deny license allow-list is active: after adding any dependency, run `cargo deny check` immediately (Task 6 is the only task that adds one).
- Commit style: plain imperative subject, no `feat:` prefix (repo convention), no AI attribution anywhere, `Closes #N` in body so the issue auto-closes on push.
- Update `README.md` features list in the same commit as the feature.
- Heavy non-cargo binaries run via `hotrun`; plain `cargo` commands already join `buildwork.slice` via the PATH shim.

---

### Task 1: Extract and save images (issue #1)

**Files:**
- Modify: `src/models.rs` (struct + field)
- Modify: `src/extractor.rs` (new section after links, ~line 52)
- Modify: `src/output/text.rs`, `src/output/html.rs`
- Modify: `README.md`

**Interfaces:**
- Produces: `models::Image { url: String, alt: String }`, `ScrapingResult.images: Vec<Image>` — Tasks 2–4 render/filter this field.

- [ ] **Step 1: Add the model** — `src/models.rs`, after `MetaTag`:

```rust
#[derive(Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub alt: String,
}
```

And inside `ScrapingResult`, after `pub meta_tags: Vec<MetaTag>,`:

```rust
    pub images: Vec<Image>,
```

- [ ] **Step 2: Initialise it** — `src/extractor.rs`, in the `ScrapingResult { ... }` literal add:

```rust
        images: Vec::new(),
```

- [ ] **Step 3: Extract images** — `src/extractor.rs`, after the links loop (after the `for (i, link)` block ends, ~line 52):

```rust
    // Get all images
    println!("\nImages found:");
    let img_selector = Selector::parse("img")?;
    let mut unique_images = HashSet::new();

    for img in document.select(&img_selector) {
        if let Some(src) = img.value().attr("src") {
            if unique_images.insert(src.to_string()) {
                let alt = img.value().attr("alt").unwrap_or("").to_string();
                println!("{} (alt: {})", src, alt);
                result.images.push(Image {
                    url: src.to_string(),
                    alt,
                });
            }
        }
    }

    if unique_images.is_empty() {
        println!("No images found");
    }
```

- [ ] **Step 4: Render in text** — `src/output/text.rs`, after the meta-tags block, before the metrics block:

```rust
    writeln!(file, "\nImages found:")?;
    if result.images.is_empty() {
        writeln!(file, "No images found")?;
    } else {
        for image in &result.images {
            writeln!(file, "{} (alt: {})", image.url, image.alt)?;
        }
    }
```

- [ ] **Step 5: Render in HTML** — `src/output/html.rs`, after the meta-tags section, before the Metrics section:

```rust
    // Images section
    writeln!(file, "  <h2>Images Found ({})</h2>", result.images.len())?;
    if result.images.is_empty() {
        writeln!(file, "  <p>No Images Found</p>")?;
    } else {
        writeln!(file, "  <ul class=\"images\">")?;
        for image in &result.images {
            writeln!(
                file,
                "    <li><a href=\"{}\">{}</a> (alt: {})</li>",
                image.url, image.url, image.alt
            )?;
        }
        writeln!(file, "  </ul>")?;
    }
```

JSON needs nothing — serde picks up the new field.

- [ ] **Step 6: Verify**

```bash
cargo build && target/debug/web_scraper --url https://www.rust-lang.org --format json --output /tmp/img_test
python3 -c "import json; print(json.load(open('/tmp/img_test.json'))['images'][:3])"
```

Expected: a non-empty list of `{url, alt}` objects.

- [ ] **Step 7: README + commit** — add `- Extracts image URLs with alt text` to the features list, then:

```bash
cargo fmt && cargo clippy --all-targets
git add -A && git commit -m 'Add image extraction

Closes #1'
git push origin main
```

---

### Task 2: CSV and XML output formats (issue #6)

**Files:**
- Create: `src/output/csv.rs`, `src/output/xml.rs`
- Modify: `src/output/mod.rs`, `src/main.rs`, `README.md`

**Interfaces:**
- Consumes: full `ScrapingResult` including `images` (Task 1) and `metrics`.
- Produces: `output::csv::save(&ScrapingResult, &str)`, `output::xml::save(&ScrapingResult, &str)` — same signature as the existing three.

- [ ] **Step 1: Write failing escape tests** — bottom of the new `src/output/csv.rs` (create the file with just this for now):

```rust
#[cfg(test)]
mod tests {
    use super::esc;

    #[test]
    fn quotes_and_doubles_inner_quotes() {
        assert_eq!(esc(r#"say "hi", ok"#), r#""say ""hi"", ok""#);
    }
}
```

Run: `cargo test` — Expected: FAIL (`esc` not found).

- [ ] **Step 2: Implement CSV** — `src/output/csv.rs`, above the tests:

```rust
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
```

Run: `cargo test` — Expected: PASS.

- [ ] **Step 3: Implement XML** — create `src/output/xml.rs`:

```rust
use crate::models::ScrapingResult;
use std::{fs::File, io::Write, path::Path};

// Order matters: escape & first or you double-escape the others
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
        writeln!(file, "    <link url=\"{}\">{}</link>", esc(&link.url), esc(&link.text))?;
    }
    writeln!(file, "  </links>")?;
    writeln!(file, "  <headers>")?;
    for header in &result.headers {
        writeln!(file, "    <header level=\"{}\">{}</header>", header.level, esc(&header.text))?;
    }
    writeln!(file, "  </headers>")?;
    writeln!(file, "  <meta_tags>")?;
    for meta in &result.meta_tags {
        writeln!(file, "    <meta name=\"{}\">{}</meta>", esc(&meta.name), esc(&meta.content))?;
    }
    writeln!(file, "  </meta_tags>")?;
    writeln!(file, "  <images>")?;
    for image in &result.images {
        writeln!(file, "    <image alt=\"{}\">{}</image>", esc(&image.alt), esc(&image.url))?;
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
```

- [ ] **Step 4: Register formats** — `src/output/mod.rs`:

```rust
pub mod csv;
pub mod html;
pub mod json;
pub mod text;
pub mod xml;
```

`src/main.rs`: replace the extension `match` (~line 15) with:

```rust
    let ext = match config.output_format.as_str() {
        "json" | "html" | "csv" | "xml" => config.output_format.as_str(),
        _ => "txt",
    };
    let output_path = format!("{}.{}", config.output_file, ext);
```

And extend the save `match`:

```rust
        match config.output_format.as_str() {
            "json" => output::json::save(&result, &output_path)?,
            "html" => output::html::save(&result, &output_path)?,
            "csv" => output::csv::save(&result, &output_path)?,
            "xml" => output::xml::save(&result, &output_path)?,
            _ => output::text::save(&result, &output_path)?,
        }
```

- [ ] **Step 5: Verify**

```bash
cargo test && cargo build
target/debug/web_scraper --url https://example.com --format csv --output /tmp/fmt_test && cat /tmp/fmt_test.csv
target/debug/web_scraper --url https://example.com --format xml --output /tmp/fmt_test && head -8 /tmp/fmt_test.xml
python3 -c "import csv; print(list(csv.reader(open('/tmp/fmt_test.csv'))))"  # stdlib parser accepts it = quoting is right
python3 -c "import xml.dom.minidom as x; x.parse('/tmp/fmt_test.xml'); print('XML OK')"
```

- [ ] **Step 6: README + commit** — update `- Saves output in multiple formats (text, JSON, HTML, CSV, XML)` and the Output Files list, then:

```bash
cargo fmt && cargo clippy --all-targets
git add -A && git commit -m 'Add CSV and XML output formats

Closes #6'
git push origin main
```

---

### Task 3: Filtering options (issue #4)

**Files:**
- Modify: `src/cli.rs`, `src/models.rs`, `src/main.rs`, `README.md`

**Interfaces:**
- Produces: `Config.filter: Option<String>`, `ScrapingResult::apply_filter(&mut self, keyword: &str)`.

- [ ] **Step 1: Failing test** — bottom of `src/models.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ScrapingResult {
        ScrapingResult {
            url: "https://example.com".into(),
            title: None,
            links: vec![
                Link { text: "Rust docs".into(), url: "https://doc.rust-lang.org".into() },
                Link { text: "News".into(), url: "https://example.com/news".into() },
            ],
            headers: vec![Header { level: 1, text: "Why Rust".into() }],
            meta_tags: Vec::new(),
            images: Vec::new(),
            metrics: Metrics::default(),
        }
    }

    #[test]
    fn filter_is_case_insensitive_and_checks_text_and_url() {
        let mut result = sample();
        result.apply_filter("RUST");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.headers.len(), 1);
        assert_eq!(result.links[0].text, "Rust docs");
    }
}
```

Run: `cargo test filter` — Expected: FAIL (`apply_filter` not found).

- [ ] **Step 2: Implement** — `src/models.rs`, in (or as) the `impl ScrapingResult` block:

```rust
impl ScrapingResult {
    /// Keep only entries whose text or URL contains `keyword` (case-insensitive).
    pub fn apply_filter(&mut self, keyword: &str) {
        let kw = keyword.to_lowercase();
        let hit = |s: &str| s.to_lowercase().contains(&kw);
        self.links.retain(|l| hit(&l.text) || hit(&l.url));
        self.headers.retain(|h| hit(&h.text));
        self.meta_tags.retain(|m| hit(&m.name) || hit(&m.content));
        self.images.retain(|i| hit(&i.alt) || hit(&i.url));
    }
}
```

Run: `cargo test filter` — Expected: PASS.

- [ ] **Step 3: CLI flag** — `src/cli.rs`: add `pub filter: Option<String>,` to `Config`, `let mut filter = None;` to `parse()`, this arm to the `match`:

```rust
                "--filter" if i + 1 < args.len() => {
                    filter = Some(args[i + 1].clone());
                    i += 1;
                }
```

and `filter,` to the final `Self { ... }`.

- [ ] **Step 4: Wire it** — `src/main.rs`, right after the `result.metrics = ...` assignment:

```rust
        if let Some(keyword) = &config.filter {
            result.apply_filter(keyword);
        }
```

- [ ] **Step 5: Verify**

```bash
cargo build
target/debug/web_scraper --url https://example.com --filter iana --output /tmp/filter_test
grep -c '^' /tmp/filter_test.txt   # small file; links section should contain only the iana.org link
```

- [ ] **Step 6: README + commit** — document `--filter <keyword>` under Additional Options:

```bash
cargo fmt && cargo clippy --all-targets
git add -A && git commit -m 'Add keyword filtering for scraped elements

Closes #4'
git push origin main
```

---

### Task 4: Crawling (issue #5)

The big one. Two halves: (a) outputs learn to render `&[ScrapingResult]`, (b) new `crawler.rs` walks same-host links breadth-first to `--depth N`.

**Files:**
- Create: `src/crawler.rs`
- Modify: `src/cli.rs`, `src/main.rs`, all five `src/output/*.rs`, `README.md`

**Interfaces:**
- Consumes: `extractor::extract`, `Config.delay_ms`, `Config.filter`.
- Produces: `crawler::crawl(client: &reqwest::Client, start: &str, depth: u32, delay_ms: u64) -> Result<Vec<ScrapingResult>, Box<dyn Error>>`; every `output::*::save` now takes `results: &[ScrapingResult]`. Task 5 rewrites only the inside of `crawl`.

- [ ] **Step 1: Output refactor — signatures.** In all five `src/output/*.rs`, change:

```rust
pub fn save(results: &[ScrapingResult], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
```

- `json.rs`: body stays one line — `to_string_pretty(results)?` (output becomes a JSON **array**; note this in README).
- `text.rs`: wrap the existing body (everything between `File::create` and `Ok(())`) in:

```rust
    for (i, result) in results.iter().enumerate() {
        writeln!(file, "===== Page {}: {} =====\n", i + 1, result.url)?;
        // existing body here, unchanged (it already starts by printing the url/title)
    }
```

- `csv.rs`: same loop, no banner needed — the existing `url,...` row per page already separates pages. Move the `writeln!(file, "kind,field1,field2")?;` header **above** the loop so it prints once.
- `xml.rs`: print `<?xml ...?>` and a new root `<scraping_results>` once, loop the existing `<scraping_result>` block per page, close `</scraping_results>`.
- `html.rs`: keep the `<head>` and footer as-is; extract everything from `// Page title` (line 37) through the meta/images/metrics sections (line ~97) into:

```rust
fn write_page(file: &mut File, result: &ScrapingResult) -> Result<(), Box<dyn std::error::Error>> {
    // moved lines go here verbatim
    Ok(())
}
```

and call it from `save` in a loop with an `<hr>` between pages:

```rust
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            writeln!(file, "  <hr>")?;
        }
        write_page(&mut file, result)?;
    }
```

- [ ] **Step 2: The crawler** — create `src/crawler.rs`:

```rust
use crate::extractor;
use crate::models::{Metrics, ScrapingResult};
use reqwest::{Client, Url};
use scraper::Html;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Only follow http(s) links on the same host as the start URL.
fn in_scope(start: &Url, candidate: &Url) -> bool {
    candidate.scheme().starts_with("http") && candidate.host() == start.host()
}

async fn fetch_page(client: &Client, url: &Url) -> Option<ScrapingResult> {
    let fetch_start = Instant::now();
    let response = client.get(url.clone()).send().await.ok()?;
    if !response.status().is_success() {
        eprintln!("Skipping {} ({})", url, response.status());
        return None;
    }
    let body = response.text().await.ok()?;
    let fetch_time_ms = fetch_start.elapsed().as_millis();

    let parse_start = Instant::now();
    let document = Html::parse_document(&body);
    let mut result = extractor::extract(url.as_str(), &document).ok()?;
    result.metrics = Metrics {
        fetch_time_ms,
        parse_time_ms: parse_start.elapsed().as_millis(),
        page_size_bytes: body.len(),
    };
    Some(result)
}

pub async fn crawl(
    client: &Client,
    start: &str,
    depth: u32,
    delay_ms: u64,
) -> Result<Vec<ScrapingResult>, Box<dyn std::error::Error>> {
    let start_url = Url::parse(start)?;
    let mut visited = HashSet::from([start_url.to_string()]);
    let mut frontier = vec![start_url.clone()];
    let mut results = Vec::new();

    for level in 0..=depth {
        let mut next = Vec::new();
        for url in frontier.drain(..) {
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            println!("\n[depth {}] Fetching {}", level, url);
            let Some(result) = fetch_page(client, &url).await else {
                continue;
            };
            if level < depth {
                for link in &result.links {
                    if let Ok(mut abs) = url.join(&link.url) {
                        abs.set_fragment(None); // #section variants are the same page
                        if in_scope(&start_url, &abs) && visited.insert(abs.to_string()) {
                            next.push(abs);
                        }
                    }
                }
            }
            results.push(result);
        }
        frontier = next;
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::in_scope;
    use reqwest::Url;

    #[test]
    fn scope_allows_same_host_rejects_foreign_and_mailto() {
        let start = Url::parse("https://example.com/a").unwrap();
        let same = start.join("/deep/page").unwrap();
        let foreign = Url::parse("https://other.org/x").unwrap();
        let mailto = Url::parse("mailto:a@b.c").unwrap();
        assert!(in_scope(&start, &same));
        assert!(!in_scope(&start, &foreign));
        assert!(!in_scope(&start, &mailto));
    }
}
```

Run: `cargo test scope` — Expected: PASS. (Write the test first if you want the red step: it fails to compile until `in_scope` exists.)

- [ ] **Step 3: Rewire main** — `src/main.rs` becomes:

```rust
mod cli;
mod crawler;
mod extractor;
mod models;
mod output;

use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = cli::Config::parse();
    println!("Fetching content from: {}", config.url);

    let ext = match config.output_format.as_str() {
        "json" | "html" | "csv" | "xml" => config.output_format.as_str(),
        _ => "txt",
    };
    let output_path = format!("{}.{}", config.output_file, ext);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut results = crawler::crawl(&client, &config.url, config.depth, config.delay_ms).await?;

    if let Some(keyword) = &config.filter {
        for result in &mut results {
            result.apply_filter(keyword);
        }
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
```

Notes: the pre-request `--delay` sleep and the error-file-on-failure block both move into/are replaced by crawler behaviour (delay before every request; failed pages are skipped with a stderr line, and an all-failed run exits 1). `File`/`Write`/`Path` imports in main.rs go away.

- [ ] **Step 4: CLI flag** — `src/cli.rs`: add `pub depth: u32,` to `Config`, `let mut depth: u32 = 0;` to `parse()`, arm:

```rust
                "--depth" if i + 1 < args.len() => {
                    depth = args[i + 1].parse().unwrap_or(0);
                    i += 1;
                }
```

and `depth,` in the final `Self { ... }`.

- [ ] **Step 5: Verify**

```bash
cargo test && cargo build
target/debug/web_scraper --url https://example.com --depth 1 --output /tmp/crawl_test
```

Expected: 2 pages (example.com links only to www.iana.org — different host, so depth 1 finds 0 same-host links; output says `1 page(s)`). Then a real multi-page check:

```bash
target/debug/web_scraper --url https://www.rust-lang.org --depth 1 --delay 500 --format json --output /tmp/crawl_test
python3 -c "import json; d=json.load(open('/tmp/crawl_test.json')); print(len(d), [p['url'] for p in d[:5]])"
```

Expected: >1 pages, all on www.rust-lang.org. Keep `--delay 500` — politeness.

- [ ] **Step 6: README + commit** — document `--depth N` (same-host, breadth-first, 0 = single page) and the JSON-array format change:

```bash
cargo fmt && cargo clippy --all-targets
git add -A && git commit -m 'Add same-host breadth-first crawling

Outputs now render a list of pages; JSON becomes an array.
Failed pages are skipped instead of aborting the run.

Closes #5'
git push origin main
```

---

### Task 5: Concurrency (issue #8)

Upgrade the crawler's per-level loop: fetch every URL in a depth level concurrently, capped by a semaphore. `visited`/`next` bookkeeping stays single-threaded in the collect loop — no locks needed.

**Files:**
- Modify: `src/crawler.rs`, `src/cli.rs`, `README.md`

**Interfaces:**
- Consumes: `fetch_page`, `in_scope` from Task 4 (unchanged).
- Produces: `crawl(client, start, depth, delay_ms, concurrency: usize)` — one added parameter; `Config.concurrency: usize` (default 4).

- [ ] **Step 1: CLI flag** — `src/cli.rs`: add `pub concurrency: usize,` to `Config`, `let mut concurrency: usize = 4;`, arm:

```rust
                "--concurrency" if i + 1 < args.len() => {
                    concurrency = args[i + 1].parse().unwrap_or(4).max(1);
                    i += 1;
                }
```

and `concurrency,` in `Self { ... }`.

- [ ] **Step 2: Concurrent level loop** — `src/crawler.rs`: add imports and replace `crawl`:

```rust
use std::sync::Arc;
use tokio::{sync::Semaphore, task::JoinSet};
```

```rust
pub async fn crawl(
    client: &Client,
    start: &str,
    depth: u32,
    delay_ms: u64,
    concurrency: usize,
) -> Result<Vec<ScrapingResult>, Box<dyn std::error::Error>> {
    let start_url = Url::parse(start)?;
    let mut visited = HashSet::from([start_url.to_string()]);
    let mut frontier = vec![start_url.clone()];
    let mut results = Vec::new();
    let semaphore = Arc::new(Semaphore::new(concurrency));

    for level in 0..=depth {
        let mut tasks = JoinSet::new();
        for url in frontier.drain(..) {
            let client = client.clone(); // reqwest::Client is an Arc internally — cheap
            let semaphore = semaphore.clone();
            tasks.spawn(async move {
                let _permit = semaphore.acquire_owned().await.ok()?;
                if delay_ms > 0 {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
                println!("\n[depth {}] Fetching {}", level, url);
                fetch_page(&client, &url).await
            });
        }

        let mut next = Vec::new();
        while let Some(joined) = tasks.join_next().await {
            let Ok(Some(result)) = joined else { continue };
            if level < depth {
                let base = Url::parse(&result.url)?;
                for link in &result.links {
                    if let Ok(mut abs) = base.join(&link.url) {
                        abs.set_fragment(None);
                        if in_scope(&start_url, &abs) && visited.insert(abs.to_string()) {
                            next.push(abs);
                        }
                    }
                }
            }
            results.push(result);
        }
        frontier = next;
    }
    Ok(results)
}
```

- [ ] **Step 3: Pass it through** — `src/main.rs`, update the call:

```rust
    let mut results =
        crawler::crawl(&client, &config.url, config.depth, config.delay_ms, config.concurrency).await?;
```

- [ ] **Step 4: Verify**

```bash
cargo test && cargo build
time target/debug/web_scraper --url https://www.rust-lang.org --depth 1 --concurrency 1 --output /tmp/c1
time target/debug/web_scraper --url https://www.rust-lang.org --depth 1 --concurrency 8 --output /tmp/c8
```

Expected: same page count, `--concurrency 8` wall-clock clearly lower. Extractor `println!`s will interleave between pages — cosmetic, the saved files are ordered.

Gotcha to know, not to fix: `scraper::Html` is not `Send`, so it must never be held across an `.await`. `fetch_page` already parses *after* its last await — that's why the spawned future stays `Send`. If you reorder it and the compiler screams about `Send`, that's what happened.

- [ ] **Step 5: README + commit** — document `--concurrency N` (default 4; combine with `--delay` for politeness):

```bash
cargo fmt && cargo clippy --all-targets
git add -A && git commit -m 'Fetch crawl levels concurrently

Semaphore-capped tokio tasks per depth level, default 4.

Closes #8'
git push origin main
```

---

### Task 6: Screenshots (issue #7)

Optional `--screenshot` flag: after scraping, drive headless Chromium over every scraped URL and save PNGs. Only task with a new dependency.

**Files:**
- Create: `src/screenshot.rs`
- Modify: `Cargo.toml`, `src/cli.rs`, `src/main.rs`, `README.md`

**Interfaces:**
- Consumes: `results` (for the URL list), `config.output_file` (PNG base name).
- Produces: `screenshot::capture(urls: &[String], output_base: &str) -> Result<(), Box<dyn Error>>` — blocking, call via `spawn_blocking`.

- [ ] **Step 1: Dependency + license gate** — `Cargo.toml`:

```toml
headless_chrome = "1"
```

Run **immediately**: `cargo deny check` — Expected: `licenses ok`. If a transitive license fails the allow-list, stop and extend `deny.toml` deliberately (one-line `allow` entry with a comment) before writing any code.

- [ ] **Step 2: Browser binary — no system install.** This machine has no system Chrome (and the rule is: don't install one). Point the crate at Playwright's bundled Chromium via the `CHROME` env var, which `headless_chrome` respects for auto-detection… verify the path exists first:

```bash
ls ~/.cache/ms-playwright/chromium-*/chrome-linux/chrome
export CHROME=$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux/chrome | tail -1)
```

If the glob is empty, find the binary with `find ~/.cache/ms-playwright -name chrome -o -name headless_shell` and export that. Put the working `export` in your shell profile if you'll use `--screenshot` often; document it in README regardless.

- [ ] **Step 3: The module** — create `src/screenshot.rs`:

```rust
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::{Browser, LaunchOptions};
use std::{fs, path::PathBuf};

/// Blocking: call from async code via tokio::task::spawn_blocking.
pub fn capture(urls: &[String], output_base: &str) -> Result<(), Box<dyn std::error::Error>> {
    let launch_options = LaunchOptions {
        path: std::env::var("CHROME").ok().map(PathBuf::from),
        ..Default::default()
    };
    let browser = Browser::new(launch_options)?;

    for (i, url) in urls.iter().enumerate() {
        let tab = browser.new_tab()?;
        tab.navigate_to(url)?.wait_until_navigated()?;
        let png = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
        let path = format!("{}_{}.png", output_base, i + 1);
        fs::write(&path, png)?;
        println!("Screenshot saved: {}", path);
    }
    Ok(())
}
```

Sequential on purpose — one browser, one tab at a time. Chromium is the heavy part, not our loop.

- [ ] **Step 4: Flag + wiring** — `src/cli.rs`: add `pub screenshot: bool,`, `let mut screenshot = false;`, arm (note: no `i += 1`, it takes no value):

```rust
                "--screenshot" => screenshot = true,
```

and `screenshot,` in `Self { ... }`.

`src/main.rs`: add `mod screenshot;` to the module list, and after the save `match` / before the final `println!`:

```rust
    if config.screenshot {
        let urls: Vec<String> = results.iter().map(|r| r.url.clone()).collect();
        let base = config.output_file.clone();
        tokio::task::spawn_blocking(move || {
            screenshot::capture(&urls, &base).map_err(|e| format!("Screenshot capture failed: {e}"))
        })
        .await??;
    }
```

(The `map_err` to `String` happens *inside* the closure because `Box<dyn Error>` isn't `Send` and so can't cross the thread boundary; the double `??` unwraps the JoinError, then the `String` error, which `?` converts back into `Box<dyn Error>` for `main`.)

- [ ] **Step 5: Verify** — Chromium is heavy; run it via `hotrun` per the machine rules:

```bash
cargo build
hotrun target/debug/web_scraper --url https://example.com --screenshot --output /tmp/shot_test
file /tmp/shot_test_1.png
```

Expected: `PNG image data, 1920 x 1080` (or similar). Also verify graceful failure: `unset CHROME` in a subshell and confirm you get the "Screenshot capture failed" message, not a panic.

- [ ] **Step 6: README + commit** — document `--screenshot`, the `CHROME` env var requirement, and that PNGs are named `<output>_<n>.png`:

```bash
cargo fmt && cargo clippy --all-targets
git add -A && git commit -m 'Add page screenshots via headless Chromium

Requires a Chromium binary; honours CHROME env var so
Playwright-bundled builds work without a system install.

Closes #7'
git push origin main
```

---

## After each task

1. `cargo fmt && cargo clippy --all-targets && cargo test` — the push gate runs all of these plus audit/deny.
2. Push; confirm the issue auto-closed: `gh issue view <N> --json state`.
3. If the gate rejects on something unrelated (new advisory), fix it in its own commit first.

## Order rationale

Images (#1) is a warm-up that later tasks render. CSV/XML (#6) lands while `save()` still takes a single result — the Task 4 refactor then updates all five formats at once instead of retrofitting new ones. Filtering (#4) is trivial pre-crawl, painful mid-crawl. Crawling (#5) before concurrency (#8): get the breadth-first logic correct sequentially, then parallelise a working thing. Screenshots (#7) last — only new dependency, zero coupling to the rest.
