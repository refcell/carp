use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// JWT configuration
    pub jwt: JwtConfig,
    /// File upload configuration
    pub upload: UploadConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub supabase_url: String,
    pub supabase_key: String,
    pub supabase_jwt_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub max_file_size: u64,
    pub allowed_mime_types: Vec<String>,
    pub storage_bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        let config = Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3001".to_string())
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid PORT value"))?,
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173,https://carp.refcell.org".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            database: DatabaseConfig {
                supabase_url: env::var("SUPABASE_URL")
                    .map_err(|_| anyhow::anyhow!("SUPABASE_URL is required"))?,
                supabase_key: env::var("SUPABASE_SERVICE_ROLE_KEY")
                    .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY is required"))?,
                supabase_jwt_secret: env::var("SUPABASE_JWT_SECRET")
                    .map_err(|_| anyhow::anyhow!("SUPABASE_JWT_SECRET is required"))?,
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                    // Generate a random secret if not provided (dev only)
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    (0..32).map(|_| rng.gen::<u8>()).map(|b| format!("{:02x}", b)).collect()
                }),
                expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
            },
            upload: UploadConfig {
                max_file_size: env::var("MAX_FILE_SIZE")
                    .unwrap_or_else(|_| "104857600".to_string()) // 100MB
                    .parse()
                    .unwrap_or(104_857_600),
                allowed_mime_types: vec![
                    "application/gzip".to_string(),
                    "application/x-gzip".to_string(),
                    "application/tar+gzip".to_string(),
                    "application/zip".to_string(),
                ],
                storage_bucket: "agent-packages".to_string(),
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: env::var("RATE_LIMIT_RPM")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                burst_size: env::var("RATE_LIMIT_BURST")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
        };

        Ok(config)
    }

    /// Load configuration from environment variables with serverless-friendly defaults  
    pub fn from_env_or_defaults() -> anyhow::Result<Self> {
        // Load .env file if it exists (not available in serverless)
        let _ = dotenvy::dotenv();

        let config = Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3001".to_string())
                    .parse()
                    .unwrap_or(3001),
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            database: DatabaseConfig {
                supabase_url: env::var("SUPABASE_URL")
                    .map_err(|_| anyhow::anyhow!("SUPABASE_URL is required"))?,
                supabase_key: env::var("SUPABASE_SERVICE_ROLE_KEY")
                    .map_err(|_| anyhow::anyhow!("SUPABASE_SERVICE_ROLE_KEY is required"))?,
                supabase_jwt_secret: env::var("SUPABASE_JWT_SECRET")
                    .map_err(|_| anyhow::anyhow!("SUPABASE_JWT_SECRET is required"))?,
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .map_err(|_| anyhow::anyhow!("JWT_SECRET is required in production"))?,
                expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
            },
            upload: UploadConfig {
                max_file_size: env::var("MAX_FILE_SIZE")
                    .unwrap_or_else(|_| "104857600".to_string()) // 100MB  
                    .parse()
                    .unwrap_or(104_857_600),
                allowed_mime_types: vec![
                    "application/gzip".to_string(),
                    "application/x-gzip".to_string(),
                    "application/tar+gzip".to_string(),
                    "application/zip".to_string(),
                ],
                storage_bucket: env::var("STORAGE_BUCKET")
                    .unwrap_or_else(|_| "agent-packages".to_string()),
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: env::var("RATE_LIMIT_RPM")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                burst_size: env::var("RATE_LIMIT_BURST")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
        };

        Ok(config)
    }
}