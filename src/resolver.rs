use regex::RegexBuilder;
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

pub async fn resolve_url(url: &str, prime: &bool) -> Result<String, reqwest::Error> {
    #[cfg(feature = "memcache")]
    {
        let stream =
            TcpStream::connect("mcache.iavian.net:11211").expect("Failed to create stream");
        let mut cache = Protocol::new(AllowStdIo::new(stream));
        let key = format!("1-1{}", &url);
        if !prime {
            let value = cache.get(&key).await.unwrap_or_default();
            let value = String::from_utf8_lossy(&value);
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
        let resolved_url = _resolve_meta(&url).await?;
        if let Ok(_) = cache.set(&key, resolved_url.as_bytes(), 0).await {
            println!("Memcache set for url {} - {}", url, resolved_url);
        }
        Ok(resolved_url)
    }

    #[cfg(not(feature = "memcache"))]
    {
        _resolve_meta(&url).await
    }
}

async fn _resolve_meta(purl: &str) -> Result<String, reqwest::Error> {
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
    let proxy = reqwest::Proxy::http("http://proxy.iavian.net:38080")?;
    let client = ClientBuilder::new()
        .proxy(proxy)
        .timeout(Duration::new(20, 0))
        .redirect(Policy::limited(10))
        .brotli(true)
        .gzip(true)
        .default_headers(headers)
        .build()?;
    let resp = client.get(purl).send().await?;
    println!("Server code {}", resp.status());
    let url = { resp.url().to_owned() };
    let body = { resp.text().await };
    let redirect = match body {
        Ok(body) => {
            let reg = RegexBuilder::new(r##"(?:http-equiv="refresh".*?)?content="\d+;url=(.*?)"(?:.*?http-equiv="refresh")?"##).case_insensitive(true).build().unwrap();
            match reg.captures(&body) {
                Some(caps) => String::from(caps.get(1).unwrap().as_str()),
                None => url.to_string(),
            }
        }
        Err(_) => url.to_string(),
    };
    let resolved_url = clean_query(&redirect).unwrap_or(String::from(purl));
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
