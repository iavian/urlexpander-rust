use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use reqwest::{header, Url};
use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "memcache")]
use futures::io::AllowStdIo;
#[cfg(feature = "memcache")]
use memcache_async::ascii::Protocol;
#[cfg(feature = "memcache")]
use std::net::TcpStream;

pub async fn resolve(url: &str, prime: &bool) -> Result<String, reqwest::Error> {
    #[cfg(feature = "memcache")]
    {
        let stream =
            TcpStream::connect("memcached.iavian.net:11211").expect("Failed to create stream");
        let mut cache = Protocol::new(AllowStdIo::new(stream));
        let key = format!("z_{}", &url);
        if !prime {
            let value = cache.get(&key).await.unwrap_or_default();
            let value = String::from_utf8_lossy(&value);
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
        let resolved_url = _resolve(&url).await?;
        let _ = cache.set(&key, resolved_url.as_bytes(), 0).await;
        Ok(resolved_url)
    }

    #[cfg(not(feature = "memcache"))]
    {
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
        .timeout(Duration::new(20, 0))
        .redirect(Policy::limited(10))
        .default_headers(headers)
        .build()?;
    let resp = client.get(url).send().await?;
    let url = resp.url();
    let resolved_url = clean_query(url.as_str()).unwrap_or(String::from(url.as_ref()));
    Ok(resolved_url)
}

fn clean_query(url: &str) -> Option<String> {
    let mut url = match Url::parse(url) {
        Ok(url) => url,
        Err(_) => return None,
    };

    let params: HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .filter(|x| !x.0.contains("utm_"))
        .collect();

    url.set_query(None);
    for (k, v) in params.into_iter() {
        url.query_pairs_mut().append_pair(&k, &v);
    }
    Some(url.into())
}
