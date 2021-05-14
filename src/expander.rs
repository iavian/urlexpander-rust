
use reqwest::header;
use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use std::time::Duration;

pub async fn resolve(url: &str) -> Result<String, reqwest::Error> {
   let mut headers = header::HeaderMap::new();
   headers.insert(
      "Referer",
      header::HeaderValue::from_static("https://google.com"),
   );
   let client = ClientBuilder::new()
       .timeout(Duration::new(5, 0))
       .redirect(Policy::limited(4))
       .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Safari/605.1.15")
       .default_headers(headers)
       .build()?;
   let resp = client.get(url).send().await?;
   let url = resp.url();
   let resolved_url = String::from(url.as_ref());
   Ok(resolved_url)
}
