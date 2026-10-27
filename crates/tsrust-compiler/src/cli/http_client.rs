//! Explicit bounded network acquisition; never called by ordinary build/dev.
use std::time::Duration;

pub(super) async fn fetch_description(address: &str, headers: &[(String, String)]) -> Result<String, String> {
    let url = reqwest::Url::parse(address).map_err(|_| "Invalid HTTP description URL")?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]" | "::1"));
    if url.username() != "" || url.password().is_some() || url.fragment().is_some() || url.query().is_some()
        || !(url.scheme() == "https" || url.scheme() == "http" && loopback) {
        return Err("Use an HTTPS description URL, or HTTP on loopback, without credentials, query or fragment".into());
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
        .connect_timeout(Duration::from_secs(5)).timeout(Duration::from_secs(30))
        .user_agent(concat!("tsrust/", env!("CARGO_PKG_VERSION")))
        .build().map_err(|_| "Cannot initialize HTTP description transport")?;
    let mut request = client.get(url).header(reqwest::header::ACCEPT, "application/json");
    for (name, variable) in headers {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|_| "Invalid description header name")?;
        if matches!(name.as_str(), "host" | "content-length" | "transfer-encoding" | "connection") {
            return Err("Description headers cannot override HTTP routing or framing".into());
        }
        let secret = std::env::var(variable).map_err(|_| format!("Description header environment variable {variable} is missing or not UTF-8"))?;
        let mut value = reqwest::header::HeaderValue::from_str(&secret).map_err(|_| "Invalid description header value")?;
        value.set_sensitive(true);
        request = request.header(name, value);
    }
    let mut response = request.send().await.map_err(|_| "HTTP description request failed (connection, TLS or timeout); no output was changed")?;
    if !response.status().is_success() {
        return Err(format!("HTTP description request returned {}; redirects are not followed", response.status()));
    }
    let content = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("");
    if content.split(';').next().unwrap_or("").trim().to_ascii_lowercase() != "application/json" {
        return Err("HTTP description requires application/json".into());
    }
    if response.content_length().is_some_and(|length| length > 1_048_576) {
        return Err("HTTP API description exceeds 1 MiB".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "Cannot read HTTP description response")? {
        if bytes.len() + chunk.len() > 1_048_576 { return Err("HTTP API description exceeds 1 MiB".into()); }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| "HTTP description must be UTF-8".into())
}
