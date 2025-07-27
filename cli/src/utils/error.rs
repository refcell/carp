use std::fmt;

/// Result type alias for Carp CLI operations
pub type CarpResult<T> = Result<T, CarpError>;

/// Main error type for the Carp CLI
#[derive(Debug)]
pub enum CarpError {
    /// IO-related errors
    Io(std::io::Error),
    /// HTTP request errors
    Http(reqwest::Error),
    /// JSON serialization/deserialization errors
    Json(serde_json::Error),
    /// TOML parsing errors
    Toml(toml::de::Error),
    /// Configuration errors
    Config(String),
    /// Authentication errors
    Auth(String),
    /// API errors with status code and message
    Api { status: u16, message: String },
    /// Agent not found
    AgentNotFound(String),
    /// Invalid agent name or version
    InvalidAgent(String),
    /// Manifest parsing errors
    ManifestError(String),
    /// File system errors
    FileSystem(String),
    /// Network connectivity errors
    Network(String),
    /// Generic errors with custom message
    Other(String),
}

impl fmt::Display for CarpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CarpError::Io(e) => write!(f, "IO error: {}", e),
            CarpError::Http(e) => write!(f, "HTTP error: {}", e),
            CarpError::Json(e) => write!(f, "JSON error: {}", e),
            CarpError::Toml(e) => write!(f, "TOML error: {}", e),
            CarpError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CarpError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            CarpError::Api { status, message } => {
                write!(f, "API error ({}): {}", status, message)
            }
            CarpError::AgentNotFound(name) => write!(f, "Agent '{}' not found", name),
            CarpError::InvalidAgent(msg) => write!(f, "Invalid agent: {}", msg),
            CarpError::ManifestError(msg) => write!(f, "Manifest error: {}", msg),
            CarpError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            CarpError::Network(msg) => write!(f, "Network error: {}", msg),
            CarpError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CarpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CarpError::Io(e) => Some(e),
            CarpError::Http(e) => Some(e),
            CarpError::Json(e) => Some(e),
            CarpError::Toml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CarpError {
    fn from(err: std::io::Error) -> Self {
        CarpError::Io(err)
    }
}

impl From<reqwest::Error> for CarpError {
    fn from(err: reqwest::Error) -> Self {
        CarpError::Http(err)
    }
}

impl From<serde_json::Error> for CarpError {
    fn from(err: serde_json::Error) -> Self {
        CarpError::Json(err)
    }
}

impl From<toml::de::Error> for CarpError {
    fn from(err: toml::de::Error) -> Self {
        CarpError::Toml(err)
    }
}

impl From<zip::result::ZipError> for CarpError {
    fn from(err: zip::result::ZipError) -> Self {
        CarpError::FileSystem(format!("ZIP error: {}", err))
    }
}
