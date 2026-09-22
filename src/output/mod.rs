pub mod csv;
pub mod html;
pub mod json;
pub mod text;
pub mod xml;

/// Escape text for HTML and XML output. `&` goes first, or it would re-escape the others.
pub fn markup_esc(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// `Some(url)` if the URL is safe to put in a `href` or `src` attribute, otherwise `None`.
pub fn safe_url(url: &str) -> Option<&str> {
    let lower = url.to_ascii_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://")).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(
            markup_esc(r#"<a href="x">&'"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;"
        );
    }

    #[test]
    fn only_http_urls_become_links() {
        assert_eq!(safe_url("https://a.org/x"), Some("https://a.org/x"));
        assert_eq!(safe_url("HTTP://a.org"), Some("HTTP://a.org"));
        assert_eq!(safe_url("javascript:alert(1)"), None);
        assert_eq!(safe_url(" javascript:alert(1)"), None);
        assert_eq!(safe_url("JaVaScRiPt:alert(1)"), None);
        assert_eq!(safe_url("data:text/html, <script>"), None);
        assert_eq!(safe_url("/relative/path"), None);
    }
}
