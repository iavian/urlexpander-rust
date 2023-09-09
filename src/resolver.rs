#[cfg(feature = "memcache")]
use futures::io::AllowStdIo;
#[cfg(feature = "memcache")]
use memcache_async::ascii::Protocol;
use regex::RegexBuilder;
use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use reqwest::{header, Url};
use std::collections::HashMap;
#[cfg(feature = "memcache")]
use std::net::TcpStream;
use std::time::Duration;
use url;

use crate::service::ResolverResult;

pub async fn resolve_url(url: &str, prime: &bool) -> Result<String, reqwest::Error> {
    #[cfg(feature = "memcache")]
    {
        let stream =
            TcpStream::connect("mcache.iavian.net:11211").expect("Failed to create stream");
        let mut cache = Protocol::new(AllowStdIo::new(stream));
        let key = format!("3-{}", &url);
        if !prime {
            let value = cache.get(&key).await.unwrap_or_default();
            let value = String::from_utf8_lossy(&value);
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
        let resolved_url = _resolve_meta(&url).await?;
        let expiry = if resolved_url == url { 7200 } else { 0 };
        if let Ok(_) = cache.set(&key, resolved_url.as_bytes(), expiry).await {
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
    let client = ClientBuilder::new()
        .timeout(Duration::new(20, 0))
        .redirect(Policy::limited(10))
        .brotli(true)
        .gzip(true)
        .default_headers(headers)
        .build()?;

    let resp = client.get(purl).send().await?;
    if resp.status().is_client_error() {
        println!("Server code {} for url {}", resp.status(), purl);
        if let Some(result) = call_external(purl).await {
            return Ok(result.eurl);
        } else {
            println!("Remote failed as well for url {}", purl);
        }
    }
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

async fn call_external(purl: &str) -> Option<ResolverResult> {
    if let Ok(client) = ClientBuilder::new().timeout(Duration::new(40, 0)).build() {
        let purl = url::form_urlencoded::Serializer::new(String::from(purl)).finish();
        let purl = format!("https://open.webkit.iavian.net/resolve?url={}", purl);
        if let Ok(resp) = client.get(&purl).send().await {
            let c: Result<ResolverResult, reqwest::Error> = resp.json().await;
            match c {
                Ok(result) => return Some(result),
                Err(_) => {}
            }
        }
    }
    None
}
