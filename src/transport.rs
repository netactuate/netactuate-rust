use crate::error::{Error, Result};
use std::sync::Arc;
use url::Url;

/// HTTP method used by the internal transport abstraction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// GET.
    Get,
    /// POST.
    Post,
    /// PATCH.
    Patch,
    /// PUT.
    Put,
    /// DELETE.
    Delete,
}

impl Method {
    /// Returns the method as an HTTP token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Patch => "PATCH",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

/// A complete HTTP request handed to a transport.
#[derive(Clone, Debug)]
pub struct Request {
    /// HTTP method.
    pub method: Method,
    /// Full URL, including the API key query parameter.
    pub url: Url,
    /// Request body.
    pub body: Option<Vec<u8>>,
    /// Content type for the request body.
    pub content_type: Option<String>,
}

/// HTTP response returned by a transport.
#[derive(Clone, Debug)]
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Response body bytes.
    pub body: Vec<u8>,
}

/// Blocking transport used by the SDK clients.
pub trait Transport: Send + Sync {
    /// Sends a request and returns the raw response.
    fn send(&self, request: Request) -> Result<Response>;
}

/// Shared boxed transport.
pub type DynTransport = Arc<dyn Transport>;

/// Reqwest-backed blocking transport.
#[cfg(feature = "blocking")]
#[derive(Debug, Default)]
pub struct ReqwestTransport {
    client: reqwest::blocking::Client,
}

#[cfg(feature = "blocking")]
impl ReqwestTransport {
    /// Creates a blocking reqwest transport.
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

#[cfg(feature = "blocking")]
impl Transport for ReqwestTransport {
    fn send(&self, request: Request) -> Result<Response> {
        let mut builder = match request.method {
            Method::Get => self.client.get(request.url),
            Method::Post => self.client.post(request.url),
            Method::Patch => self.client.patch(request.url),
            Method::Put => self.client.put(request.url),
            Method::Delete => self.client.delete(request.url),
        }
        .header("Accept", "application/json");

        if let Some(content_type) = request.content_type {
            builder = builder.header("Content-Type", content_type);
        }
        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let response = builder
            .send()
            .map_err(|err| Error::Transport(redact_text(&err.to_string())))?;
        let status = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|err| Error::Transport(redact_text(&err.to_string())))?
            .to_vec();
        Ok(Response { status, body })
    }
}

/// Removes an API key from arbitrary text before it is surfaced.
///
/// `redact_url` only helps where a URL is formatted deliberately. A transport error carries the
/// URL inside a message the caller never constructed, so the key reaches the error text by a
/// path no call site is looking at. Redaction is applied to the text itself for that reason.
pub fn redact_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("key=") {
        out.push_str(&rest[..at]);
        out.push_str("key=REDACTED");
        rest = &rest[at + 4..];
        let end = rest.find(['&', ' ', ')', '"', '\'']).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Redacts API key query parameters from URLs before they are displayed.
pub fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let has_key = redacted.query_pairs().any(|(key, _)| key == "key");
    if has_key {
        let pairs: Vec<(String, String)> = redacted
            .query_pairs()
            .map(|(key, value)| {
                if key == "key" {
                    (key.into_owned(), "REDACTED".to_string())
                } else {
                    (key.into_owned(), value.into_owned())
                }
            })
            .collect();
        redacted.query_pairs_mut().clear().extend_pairs(pairs);
    }
    redacted.to_string()
}

pub(crate) fn api_key_from_env() -> Result<String> {
    std::env::var("NETACTUATE_API_KEY")
        .ok()
        .filter(|key| !key.is_empty())
        .ok_or(Error::MissingApiKey)
}

pub(crate) fn build_url(base: &Url, path: &str, api_key: &str) -> Result<Url> {
    let mut url = base.join(path.trim_start_matches('/'))?;
    url.query_pairs_mut().append_pair("key", api_key);
    Ok(url)
}
