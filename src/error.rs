use thiserror::Error;

/// Convenient result type returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type returned by all SDK operations.
#[derive(Debug, Error)]
pub enum Error {
    /// No API key was provided and `NETACTUATE_API_KEY` was not set.
    #[error("NETACTUATE_API_KEY is not set")]
    MissingApiKey,

    /// A URL could not be built from the supplied base URL or path.
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// The HTTP transport failed before a response body was available.
    #[error("transport error: {0}")]
    Transport(String),

    /// The API returned a failure that is not one of the distinguishable cases.
    #[error("API error on {method} {url}: code {status_code} / {api_code}, response: {message}")]
    Api {
        /// HTTP method.
        method: String,
        /// Redacted URL.
        url: String,
        /// HTTP status code.
        status_code: u16,
        /// Envelope code when present.
        api_code: i64,
        /// Secret-free API message.
        message: String,
    },

    /// The requested resource does not exist or is no longer visible to the account.
    #[error("not found on {method} {url}: code {status_code} / {api_code}, response: {message}")]
    NotFound {
        /// HTTP method.
        method: String,
        /// Redacted URL.
        url: String,
        /// HTTP status code.
        status_code: u16,
        /// Envelope code when present.
        api_code: i64,
        /// Secret-free API message.
        message: String,
    },

    /// The account contract does not entitle the requested capability.
    #[error(
        "contract refused on {method} {url}: code {status_code} / {api_code}, response: {message}"
    )]
    Contract {
        /// HTTP method.
        method: String,
        /// Redacted URL.
        url: String,
        /// HTTP status code.
        status_code: u16,
        /// Envelope code when present.
        api_code: i64,
        /// Secret-free API message.
        message: String,
    },

    /// A response body could not be decoded.
    #[error("decode error: {0}")]
    Decode(String),

    /// A polling wait exceeded its configured timeout before the condition it was waiting on
    /// was met.
    #[error("timed out waiting: {0}")]
    Timeout(String),

    /// JSON serialization or deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Returns true when the error is a distinguishable not found response.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Returns true when the error is a distinguishable contract gate response.
    pub fn is_contract(&self) -> bool {
        matches!(self, Self::Contract { .. })
    }

    /// Returns true when the error is a polling wait that exceeded its timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}
