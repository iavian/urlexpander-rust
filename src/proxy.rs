use std::time::Duration;

use bytes::Bytes;
use reqwest::{
    header::{self, HeaderMap},
    redirect::Policy,
    ClientBuilder,
};

pub async fn proxy_url(purl: &str) -> Result<Option<ProxyResult>, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Referer",
        header::HeaderValue::from_static("https://google.com/"),
    );
    headers.insert(
        "Accept",
        header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
    );
    headers.insert(
        "Accept-Language",
        header::HeaderValue::from_static("en-GB,en;q=0.9"),
    );
    headers.insert(
        "User-Agent",
        header::HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16."),
    );
    let client = ClientBuilder::new()
        .timeout(Duration::new(20, 0))
        .redirect(Policy::limited(10))
        .brotli(true)
        .gzip(true)
        .default_headers(headers)
        .build()?;

    let resp = client.get(purl).send().await?;
    let h = resp.headers().clone();

    if let Ok(b) = resp.bytes().await {
        let result = ProxyResult {
            bytes: b,
            headers: h,
        };
        return Ok(Some(result));
    }
    return Ok(None);
}

pub struct ProxyResult {
    pub bytes: Bytes,
    pub headers: HeaderMap,
}
