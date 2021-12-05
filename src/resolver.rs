use reqwest::header;
use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use std::time::Duration;

#[cfg(feature = "memcache")]
use futures::io::AllowStdIo;
#[cfg(feature = "memcache")]
use memcache_async::ascii::Protocol;
#[cfg(feature = "memcache")]
use std::net::TcpStream;

pub async fn resolve(url: &str) -> Result<String, reqwest::Error> {
    #[cfg(feature = "memcache")]
    {
        let stream =
            TcpStream::connect("memcached.iavian.net:11211").expect("Failed to create stream");
        let mut cache = Protocol::new(AllowStdIo::new(stream));
        let key = format!("c_{}", &url);
        let value = cache.get(&key).await.unwrap_or_default();
        let value = String::from_utf8_lossy(&value);
        if !value.is_empty() {
            return Ok(value.to_string());
        }
        let resolved_url = _resolve(&url).await?;
        let _r = cache.set(&key, resolved_url.as_bytes(), 0).await;
        Ok(resolved_url)
    }

    #[cfg(not(feature = "memcache"))] {
      _resolve(&url).await
    }
}

async fn _resolve(url: &str) -> Result<String, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Referer",
        header::HeaderValue::from_static("https://google.com"),
    );
    let client = ClientBuilder::new()
        .timeout(Duration::new(10, 0))
        .redirect(Policy::limited(4))
        .default_headers(headers)
        .build()?;
    let resp = client.get(url).send().await?;
    let url = resp.url();
    let resolved_url = String::from(url.as_ref());
    Ok(resolved_url)
}
