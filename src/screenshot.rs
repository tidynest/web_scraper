use crate::models::ScrapingResult;
use std::time::Duration;
use tokio::process::Command;

/// Save a 1920x1080 PNG of each page using Chromium's own `--screenshot` flag.
/// The binary comes from `$CHROME`. A page that times out or fails is skipped.
pub async fn capture(results: &mut [ScrapingResult], output_base: &str) -> Result<(), String> {
    let chrome = std::env::var("CHROME").map_err(|_| "set CHROME to a Chromium binary")?;

    // ponytail: One browser launch per page, sequential (~1 s each).
    // Move to a CDP client if large crawls make this the bottleneck.
    for (i, result) in results.iter_mut().enumerate() {
        let path = format!("{}_{}.png", output_base, i + 1);
        // result.url is always an absolute http(s) URL the crawler already fetched,
        // so Chromium can never read it as a command-line flag.
        let run = Command::new(&chrome)
            .args(["--headless", "--window-size=1920,1080", "--hide-scrollbars"])
            .arg(format!("--screenshot={}", path))
            .arg(&result.url)
            .kill_on_drop(true)
            .output();

        match tokio::time::timeout(Duration::from_secs(30), run).await {
            Ok(Ok(out)) if out.status.success() => result.screenshot = Some(path),
            Ok(Ok(out)) => eprintln!("Skipping screenshot of {} ({})", result.url, out.status),
            Ok(Err(err)) => return Err(format!("Cannot run {}: {}", chrome, err)),
            Err(_) => eprintln!("Skipping screenshot of {} (timed out)", result.url),
        }
    }
    Ok(())
}
