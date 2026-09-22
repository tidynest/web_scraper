use crate::extractor;
use crate::models::{Metrics, ScrapingResult};
use reqwest::{Client, Url};
use scraper::Html;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::{sync::Semaphore, task::JoinSet};

/// Only follow http(s) links on the same host as the starting URL.
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
            let client = client.clone(); // reqwest::Client is an Arc internally, cheap to clone
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
